use sqlx::PgPool;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::interval;

use crate::models::PriceUpdate;
use tokio::sync::broadcast::Sender;

pub fn spawn_price_updater(pool: PgPool, tx: Sender<PriceUpdate>) {
    tokio::spawn(async move {
        let mut timer = interval(Duration::from_secs(300));
        let client = reqwest::Client::builder()
    .user_agent("InvestWalletApp/1.0 (Contact: tu_email@ejemplo.com)")
    .build()
    .unwrap();  

        loop {
            timer.tick().await;
            println!("🔄 [Worker] Iniciando ciclo de actualización...");

            // 1. Obtenemos solo los activos que tienen un api_id asignado
            // Nota: En la macro query! debes usar un Option<String> porque la columna podría ser nula
            let assets =
                match sqlx::query!("SELECT id, api_id FROM assets WHERE api_id IS NOT NULL")
                    .fetch_all(&pool)
                    .await
                {
                    Ok(records) => records,
                    Err(e) => {
                        eprintln!("❌ [Worker] Error leyendo activos de la BD: {}", e);
                        continue; // Saltamos este ciclo y reintentamos en 5 minutos
                    }
                };

            if assets.is_empty() {
                println!("⚠️ [Worker] No hay activos con api_id para actualizar.");
                continue;
            }

            // 2. Extraemos los IDs y construimos la URL dinámicamente
            // Filtramos los nulos (aunque el SQL ya lo hizo, Rust nos pide desempaquetar el Option)
            let ids: Vec<String> = assets.into_iter().filter_map(|a| a.api_id).collect();
            let ids_query = ids.join(","); // Resultado: "bitcoin,ethereum,solana"

            let url = format!(
                "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd",
                ids_query
            );

            // 3. Consumimos la API
            match client.get(&url).send().await {
                Ok(response) => {
                    // Validamos si CoinGecko nos dio un código de error (ej. 403, 429)
                    if !response.status().is_success() {
                        let status = response.status();
                        let text_error = response.text().await.unwrap_or_default();
                        eprintln!("❌ [Worker] CoinGecko bloqueó la petición. Código HTTP: {}. Detalle: {}", status, text_error);
                        continue; // Saltamos este ciclo
                    }

                    // Ahora sí, deserializamos el JSON
                    match response.json::<HashMap<String, HashMap<String, f64>>>().await {
                        Ok(data) => {
                            // 4. Actualizamos la base de datos iterando sobre las claves obtenidas
                            for (api_id, price_data) in data {
                                if let Some(&usd_price) = price_data.get("usd") {
                                    let result = sqlx::query!(
                                        "UPDATE assets SET unit_value = $1 WHERE api_id = $2",
                                        usd_price,
                                        api_id
                                    )
                                    .execute(&pool)
                                    .await;

                                    if result.is_ok() {
                                        let update = PriceUpdate {
                                            api_id: api_id.clone(),
                                            unit_value: usd_price,
                                        };
                                        let _ = tx.send(update);

                                        println!("✅ [Worker] {} actualizado a ${}", api_id, usd_price);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            // AQUÍ mostramos el error EXACTO de Serde
                            eprintln!("❌ [Worker] Error real parseando la estructura: {}", e);
                        }
                    }
                }
                Err(e) => eprintln!("❌ [Worker] Error de red consultando la API: {}", e),
            }
        }
    });
}
