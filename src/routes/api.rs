use crate::app::AppState;
use crate::auth::admin_auth::AdminAuth;
use crate::models::Asset;
use axum::Json;
use axum::extract::State;
use axum::routing::{get};
use serde::Deserialize;
use std::collections::HashMap;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/assets", get(list_assets).post(create_asset))
        .route("/assets/update", axum::routing::patch(update_asset))
}

//Axum implementa un injector de dependencias con State, que permite inyectar el estado de la aplicacion en las rutas, para poder acceder a los assets compartidos entre todas las rutas
//Usamos el atributo #[tracing::instrument(skip_all)] para que no se loguee el estado de la aplicacion, ya que es un vector de assets que puede ser muy grande y no queremos loguearlo, tambien usamos skip_all cuando no queremos que la terminal prite datos sensibles, como contraseñas, tokens, etc.
#[tracing::instrument(skip_all)]
pub async fn list_assets(state: State<AppState>) -> Json<HashMap<i64, Asset>> {
    let assets = state.assets.lock().await; // Bloquea el mutex para acceder al vector de assets, y lo desbloquea automaticamente cuando sale del scope

    Json(assets.clone())
}

#[derive(Deserialize)]
pub struct CreateAssetRequest {
    name: String,
    unit_value: f64,
}

// Endpoint que hace el registro de nuestros assets
#[tracing::instrument(skip_all)]
pub async fn create_asset(
    _admin: AdminAuth, // Inyectamos la dependencia de AdminAuth para que solo los administradores puedan crear assets
    state: State<AppState>,
    Json(request): Json<CreateAssetRequest>,
) -> Json<Asset> {
    let mut assets = state.assets.lock().await;

    //Creamos un id de momento, pero en un futuro la hace la db.
    let new_id = assets
        .keys()
        .max()
        .cloned()
        .unwrap_or_default()
        + 1; // Genera un ID único basado en la longitud del vector de assets

    let new_asset = Asset {
        id: new_id, // Aquí deberías generar un ID único
        name: request.name,
        unit_value: request.unit_value,
    };
    assets.insert(new_id, new_asset.clone());
    Json(new_asset)
}

// actualiza el asset, pero solo si el id existe, sino devuelve un error 404
#[derive(Deserialize)]
pub struct UpdateAssetRequest {
    id: i64,
    name: Option<String>,
    unit_value: Option<f64>,
}

#[tracing::instrument(skip_all)]
pub async fn update_asset(
    _admin: AdminAuth,
    state: State<AppState>,
    Json(request): Json<UpdateAssetRequest>,
) -> Result<Json<Asset>, (axum::http::StatusCode, String)> {
    let mut assets = state.assets.lock().await;

    if let Some(asset) = assets.get_mut(&request.id) {
        if let Some(name) = request.name {
            asset.name = name;
        }
        if let Some(unit_value) = request.unit_value {
            asset.unit_value = unit_value;
        }
        Ok(Json(asset.clone()))
    } else {
        Err((
            axum::http::StatusCode::NOT_FOUND,
            "Asset not found".to_string(),
        ))
    }
}
