use crate::models::{Asset, OwnedAsset, UserRecord};
use axum::extract::FromRequestParts;
use sqlx::PgPool;
use std::convert::Infallible;

use crate::app::AppState;

pub struct Repository {
    db: PgPool,
}

//Aqui definimos las funciones que van a interactuar con la base de datos, como por ejemplo crear, leer, actualizar y eliminar assets de la base de datos. Estas funciones van a ser llamadas desde las rutas, y van a recibir como parametro el estado de la aplicacion, para poder acceder a la conexion a la base de datos.
impl From<PgPool> for Repository {
    fn from(db: PgPool) -> Self {
        Repository { db }
    }
}

impl Repository {
    //Users
    //Nota: Vamos a recibir ya el hash , el repository no se va a encargar de hashear la contraseña, eso lo hace el handler de la ruta, para que el repository solo se encargue de interactuar con la base de datos y no tenga que preocuparse por la logica de negocio.
    //Retornamos el id del usuario creado, para eso hacemos un query que inserte el usuario y retorne el id del usuario creado
    pub async fn insert_user_to_db(
        &self,
        username: &str,
        password_hash: &str,
    ) -> Result<UserRecord, crate::errors::AppError> {
        let result = sqlx::query_as!(
        UserRecord,
        "INSERT INTO users (username, password_hash) VALUES ($1, $2) RETURNING id, username, password_hash;",
        username,
        password_hash
    )
    .fetch_one(&self.db)
    .await;

        match result {
            Ok(user) => Ok(user),
            // Código 23505 = Unique Violation en PostgreSQL
            Err(sqlx::Error::Database(db_err)) if db_err.code().as_deref() == Some("23505") => {
                Err(crate::errors::AppError::UserAlreadyExists)
            }
            // Para SQLite el mensaje/código varía, o para cualquier otro error genérico:
            Err(e) => Err(crate::errors::AppError::DatabaseError(e)),
        }
    }

    pub async fn find_user_by_id(
        &self,
        user_id: i64,
    ) -> Result<Option<UserRecord>, crate::errors::AppError> {
        let result = sqlx::query_as!(
            UserRecord,
            "SELECT id, username, password_hash FROM users WHERE id = $1;",
            user_id
        )
        .fetch_optional(&self.db)
        .await;

        match result {
            Ok(user) => Ok(user),
            Err(e) => Err(crate::errors::AppError::DatabaseError(e)),
        }
    }

    pub async fn find_by_username(
        &self,
        username: &str,
    ) -> Result<Option<UserRecord>, crate::errors::AppError> {
        sqlx::query_as!(
            UserRecord,
            "SELECT id, username, password_hash FROM users WHERE username = $1;",
            username
        )
        .fetch_optional(&self.db)
        .await
        // Convertimos el sqlx::Error genérico en el error de nuestra aplicación
        .map_err(crate::errors::AppError::DatabaseError)
    }

    // Assets
    pub async fn list_assets_from_db(&self) -> sqlx::Result<Vec<Asset>> {
        //Aqui hacemos la consulta a la base de datos para obtener todos los assets, y los devolvemos como un vector de assets
        let assets = sqlx::query_as!(Asset, "SELECT id, name, unit_value, api_id FROM assets;")
            .fetch_all(&self.db)
            .await?;
        Ok(assets)
    }

    pub async fn insert_asset_to_db(
    &self,
    name: String,
    unit_value: f64,
    api_id: Option<String>, // <- Recibir el parámetro
) -> sqlx::Result<Asset> {
    let asset = sqlx::query_as!(
        Asset,
        // ¡Tienes que incluir api_id en el INSERT y en los VALUES ($3)!
        "INSERT INTO assets (name, unit_value, api_id) VALUES ($1, $2, $3) RETURNING id, name, unit_value, api_id;",
        name,
        unit_value,
        api_id
    )
    .fetch_one(&self.db)
    .await?;
    
    Ok(asset)
}

    pub async fn update_asset_to_db(
        &self,
        id: i64,
        name: Option<String>,
        unit_value: Option<f64>,
        api_id: Option<String>,
    ) -> sqlx::Result<Option<Asset>> {
        let asset = sqlx::query_as!(
            Asset,
            "UPDATE assets SET name = COALESCE($2, name), unit_value = COALESCE($3, unit_value), api_id = COALESCE($4, api_id) WHERE id = $1 RETURNING id, name, unit_value, api_id;",
            id,
            name,
            unit_value,
            api_id
        )
        .fetch_optional(&self.db)
        .await?;
        Ok(asset)
    }

    pub async fn delete_asset_from_db(&self, id: i64) -> sqlx::Result<bool> {
        let result = sqlx::query!("DELETE FROM assets WHERE id = $1;", id)
            .execute(&self.db)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    //Owned Assets
pub async fn list_owned_assets_from_db(&self, user_id: i64) -> sqlx::Result<Vec<OwnedAsset>> {
        let owned_assets = sqlx::query_as!(
            OwnedAsset,
            r#"
        SELECT 
            a.id,
            a.name,
            a.unit_value,
            SUM((a.unit_value - o.bought_for) * o.quantity_owned) AS "value_delta!",
            SUM(o.quantity_owned) AS "quantity_owned!",
            JSON_AGG(
                JSON_BUILD_OBJECT(
                    'id', o.id, -- <--- ESTA ES LA LÍNEA CLAVE QUE FALTABA
                    'bought_at', o.timestamp,
                    'bought_for', o.bought_for,
                    'quantity_bought', o.quantity_owned,
                    'value_delta', (a.unit_value - o.bought_for) * o.quantity_owned
                )
            ) AS "purchase_history!:  _"
        FROM owned_assets o
        JOIN assets a ON o.asset_id = a.id
        WHERE o.user_id = $1
        GROUP BY a.id, a.name, a.unit_value;
        "#,
            user_id
        )
        .fetch_all(&self.db)
        .await?;

        Ok(owned_assets)
    }

    pub async fn insert_owned_asset_to_db(
        &self,
        user_id: i64,
        asset_id: i64,
        quantity: f64,
        bought_for: f64,
    ) -> sqlx::Result<()> {
        sqlx::query!(
        "INSERT INTO owned_assets (user_id, asset_id, quantity_owned, bought_for) VALUES ($1, $2, $3, $4);",
        user_id,
        asset_id,
        quantity,
        bought_for
    )
    .execute(&self.db)
    .await?;
        Ok(())
    }

    pub async fn update_owned_asset_history_in_db(
        &self,
        user_id: i64,
        history_id: i64,
        quantity: f64,
        bought_for: f64,
    ) -> sqlx::Result<bool> {
        let result = sqlx::query!(
            // Eliminamos la búsqueda inútil del asset_id
            "UPDATE owned_assets SET quantity_owned = $1, bought_for = $2 WHERE id = $3 AND user_id = $4;",
            quantity,
            bought_for,
            history_id,
            user_id
        )
        .execute(&self.db)
        .await?;
        Ok(result.rows_affected() > 0)
    }
    
    pub async fn delete_owned_asset_history_from_db(
        &self,
        user_id: i64,
        history_id: i64,
    ) -> sqlx::Result<bool> {
        let result = sqlx::query!(
            "DELETE FROM owned_assets WHERE user_id = $1 AND id = $2;",
            user_id,
            history_id
        )
        .execute(&self.db)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_owned_asset_from_db(
        &self,
        user_id: i64,
        asset_id: i64,
    ) -> sqlx::Result<bool> {
        let result = sqlx::query!(
            "DELETE FROM owned_assets WHERE user_id = $1 AND asset_id = $2;",
            user_id,
            asset_id
        )
        .execute(&self.db)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}

//haremos que repository sea tambien una dependencia inyectada por el propio axon, esto lo pasamos como argumento a las funciones de las rutas, y axum se encargara de inyectar la dependencia automaticamente, para eso implementamos el trait FromRequestParts para Repository, y le decimos que el estado de la aplicacion es AppState, y que se inicializa con   AppState::new()
impl FromRequestParts<AppState> for Repository {
    type Rejection = Infallible;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self {
            db: state.db.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    // =====================================================================
    // TESTS DE USUARIOS
    // =====================================================================

    #[sqlx::test]
    async fn test_insert_user_and_unique_violation(pool: PgPool) {
        let repo = Repository::from(pool);

        // 1. Inserción exitosa
        let user = repo.insert_user_to_db("juan_perez", "hash_seguro").await.expect("Debería insertar el usuario");
        assert_eq!(user.username, "juan_perez");
        assert!(user.id > 0);

        // 2. Violación de restricción UNIQUE (Mismo username)
        let duplicate_result = repo.insert_user_to_db("juan_perez", "otro_hash").await;
        
        // Usamos matches! porque es probable que AppError no derive PartialEq
        assert!(
            matches!(duplicate_result, Err(crate::errors::AppError::UserAlreadyExists)),
            "Debería haber fallado con UserAlreadyExists por el duplicado, pero devolvió: {:?}", 
            duplicate_result
        );
    }

    #[sqlx::test(fixtures("routes/fixtures/insert_user.sql"))]
    async fn test_find_user_queries(pool: PgPool) {
        let repo = Repository::from(pool);

        // base_data.sql asume que insertó un usuario con id = 1 y username = 'testuser'
        let user_by_id = repo.find_user_by_id(1).await.expect("Error SQL").expect("El usuario no existe");
        assert_eq!(user_by_id.username, "testuser");

        let user_by_name = repo.find_by_username("testuser").await.expect("Error SQL").expect("El usuario no existe");
        assert_eq!(user_by_name.id, 1);
    }

    // =====================================================================
    // TESTS DE ACTIVOS GLOBALES (ASSETS)
    // =====================================================================

    #[sqlx::test]
    async fn test_crud_assets(pool: PgPool) {
        let repo = Repository::from(pool);

        // Crear
        let new_asset = repo.insert_asset_to_db("Ethereum".to_string(), 3000.0, Some("ethereum".to_string())).await.expect("Fallo al insertar asset");
        assert_eq!(new_asset.name, "Ethereum");

        // Leer
        let assets = repo.list_assets_from_db().await.expect("Fallo al listar assets");
        assert_eq!(assets.len(), 1);

        // Actualizar
        let updated = repo.update_asset_to_db(new_asset.id, Some("Ethereum".to_string()), Some(3200.0), Some("ethereum".to_string()))
            .await
            .expect("Fallo SQL")
            .expect("No retornó el asset actualizado");
        assert_eq!(updated.name, "Ethereum");
        assert_eq!(updated.unit_value, 3200.0);

        // Eliminar
        let deleted = repo.delete_asset_from_db(new_asset.id).await.expect("Fallo al eliminar");
        assert!(deleted, "Debería retornar true al afectar filas");
        
        let empty_list = repo.list_assets_from_db().await.unwrap();
        assert_eq!(empty_list.len(), 0);
    }

    // =====================================================================
    // TESTS DE PORTAFOLIO DE USUARIO (OWNED ASSETS)
    // =====================================================================

    #[sqlx::test(fixtures("routes/fixtures/insert_owned_asset.sql"))]
    async fn test_owned_assets_lifecycle(pool: PgPool) {
        let repo = Repository::from(pool);
        // User 1 y Asset 1 ya existen por el fixture base_data.sql

        // 1. Insertar un par de compras del mismo activo
        repo.insert_owned_asset_to_db(1, 1, 2.0, 40000.0).await.expect("Fallo compra 1");
        repo.insert_owned_asset_to_db(1, 1, 0.5, 60000.0).await.expect("Fallo compra 2");

        // 2. Probar la consulta compleja con JSON_AGG y matemáticas
        let portfolio = repo.list_owned_assets_from_db(1).await.expect("Fallo al listar portafolio");
        
        assert_eq!(portfolio.len(), 1, "Debería agrupar las dos compras en un solo activo");
        let my_bitcoin = &portfolio[0];
        
        assert_eq!(my_bitcoin.quantity_owned, 2.5); // 2.0 + 0.5
        // Asset vale 50k en base_data.sql. 
        // Compra 1: 2.0 * (50k - 40k) = +20,000
        // Compra 2: 0.5 * (50k - 60k) = -5,000
        // Total Delta: 15,000
        assert_eq!(my_bitcoin.value_delta, 15000.0);
        
        // Verifica que el JSON se estructuró correctamente en el struct
        assert_eq!(my_bitcoin.purchase_history.0.len(), 2, "Debe haber 2 historiales de compra");
    }

    #[sqlx::test(fixtures("routes/fixtures/update_owned_asset.sql"))]
    async fn test_update_and_delete_history(pool: PgPool) {
        let repo = Repository::from(pool.clone());
        // owned_asset.sql asume un historial con id = 1 para el usuario = 1

        // 1. Actualizar
        let updated = repo.update_owned_asset_history_in_db(1, 1, 5.0, 10000.0).await.expect("Fallo al actualizar");
        assert!(updated, "Debería haber modificado la fila");

        // 2. Eliminar historial específico
        let deleted_history = repo.delete_owned_asset_history_from_db(1, 1).await.expect("Fallo al eliminar historial");
        assert!(deleted_history, "Debería haber eliminado el historial");

        // Comprobar BD vacía
        let count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM owned_assets")
            .fetch_one(&pool)
            .await
            .unwrap()
            .unwrap_or(0);
        assert_eq!(count, 0);
    }
}
