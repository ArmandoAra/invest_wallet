use crate::app::AppState;
use crate::auth::admin_auth::AdminAuth;
use crate::auth::user_auth::UserAuth;
use crate::errors::AppError;
use crate::models::Asset;
use crate::repository::{Repository};
use axum::Json;
use axum::routing::get;
use serde::Deserialize;
use crate::models::PurchaseAssetRequest;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/assets", get(list_assets).post(create_asset))
        .route("/assets/update", axum::routing::patch(update_asset))
        .route("/assets/purchase", axum::routing::post(purchase_asset))
        .route("/assets/purchase/update/{asset_id}", axum::routing::patch(update_owned_asset_history))
        .route("/assets/purchase/delete/{asset_id}", axum::routing::delete(delete_owned_asset_history))
        .route("/assets/delete/{asset_id}", axum::routing::post(delete_asset))
        .route("/assets/update/{asset_id}", axum::routing::patch(update_asset))
}

//Axum implementa un injector de dependencias con State, que permite inyectar el estado de la aplicacion en las rutas, para poder acceder a los assets compartidos entre todas las rutas
//Usamos el atributo #[tracing::instrument(skip_all)] para que no se loguee el estado de la aplicacion, ya que es un vector de assets que puede ser muy grande y no queremos loguearlo, tambien usamos skip_all cuando no queremos que la terminal prite datos sensibles, como contraseñas, tokens, etc.
#[tracing::instrument(skip_all)]
async fn list_assets(
    _admin: AdminAuth,
    repository: Repository,
) -> Result<Json<Vec<Asset>>, AppError> {
    let assets = repository
        .list_assets_from_db()
        .await
        .map_err(|_| AppError::DatabaseError(sqlx::Error::RowNotFound))?; // Manejar el error adecuadamente en un caso real

    Ok(Json(assets.clone()))
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
    repository: Repository, // Inyectamos la dependencia de Repository para poder acceder a la base de datos
    Json(request): Json<CreateAssetRequest>,
) -> Result<Json<Asset>, AppError> {
    let asset = repository
        .insert_asset_to_db(request.name, request.unit_value)
        .await
        .map_err(|_| AppError::DatabaseError(sqlx::Error::RowNotFound))?; // Manejar el error adecuadamente en un caso real

    Ok(Json(asset))
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
    repository: Repository,
    Json(request): Json<UpdateAssetRequest>,
) -> Result<Json<Option<Asset>>, AppError> {
    match repository
        .update_asset_to_db(request.id, request.name, request.unit_value)
        .await
        .map_err(|_| AppError::DatabaseError(sqlx::Error::RowNotFound))?
    {
        Some(asset) => Ok(Json(Some(asset))),
        None => Err(AppError::AssetsDoesNotExist),
    }
}

pub async fn purchase_asset(
    user_auth: UserAuth,
    repository: Repository,
    Json(request): Json<PurchaseAssetRequest>,
) -> Result<(), AppError> {
    repository
        .insert_owned_asset_to_db(
            user_auth.user_id(),
            request.asset_id,
            request.quantity_owned,
            request.unit_value,
        )
        .await
        .map_err(|_| AppError::DatabaseError(sqlx::Error::RowNotFound))?; // Manejar el error adecuadamente en un caso real

    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn update_owned_asset_history(
    user_auth: UserAuth,
    repository: Repository,
    Json(request): Json<PurchaseAssetRequest>,
) -> Result<(), AppError> {
    repository
        .update_owned_asset_history_in_db(
            user_auth.user_id(),
            request.history_id, 
            request.quantity_owned,
            request.unit_value,
        )
        .await
        .map_err(|_| AppError::DatabaseError(sqlx::Error::RowNotFound))?; // Manejar el error adecuadamente en un caso real

    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn delete_owned_asset_history(
    user_auth: UserAuth,
    repository: Repository,
    axum::extract::Path(asset_id): axum::extract::Path<i64>,
) -> Result<(), AppError> {
    repository
        .delete_owned_asset_from_db(user_auth.user_id(), asset_id)
        .await
        .map_err(|_| AppError::DatabaseError(sqlx::Error::RowNotFound))?; // Manejar el error adecuadamente en un caso real

    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn delete_asset(
    _admin: AdminAuth,
    repository: Repository,
    axum::extract::Path(asset_id): axum::extract::Path<i64>,
) -> Result<(), AppError> {
    repository
        .delete_asset_from_db(asset_id)
        .await
        .map_err(|_| AppError::DatabaseError(sqlx::Error::RowNotFound))?; // Manejar el error adecuadamente en un caso real

    Ok(())
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;
    use super::*;
    // Assets 
    #[sqlx::test]
    async fn test_create_asset(pool: PgPool) {
        let repository = Repository::from(pool);
        let request = CreateAssetRequest {
            name: "Test Asset".to_string(),
            unit_value: 100.0,
        };

        let result = create_asset(AdminAuth, repository, Json(request)).await;
        assert!(result.is_ok());
        if let Ok(Json(asset)) = result {
            assert_eq!(asset.id, 1); // Assuming this is the first asset being created in the test database
            assert_eq!(asset.name, "Test Asset");
            assert_eq!(asset.unit_value, 100.0);

            insta::assert_json_snapshot!(asset);
        }
    }

    #[sqlx::test(fixtures("bitcoin_asset.sql"))] //Las fixtures son archivos SQL que se ejecutan antes de cada test, para poblar la base de datos con datos de prueba. En este caso, estamos usando la fixture bitcoin_asset.sql para poblar la base de datos con un asset llamado Bitcoin antes de ejecutar el test.
    async fn test_list_assets(pool: PgPool) {
        let repository = Repository::from(pool);
        let result = list_assets(AdminAuth, repository).await;
        assert!(result.is_ok());
        if let Ok(Json(assets)) = result {
            assert_eq!(assets.len(), 1); // Assuming the fixture inserts one asset
            assert_eq!(assets[0].name, "Bitcoin");
            assert_eq!(assets[0].unit_value, 50000.0);

            insta::assert_json_snapshot!(assets);
        }
    }

    #[sqlx::test(fixtures("bitcoin_asset.sql"))]
    async fn test_update_asset(pool: PgPool) {
        let repository = Repository::from(pool);
        let request = UpdateAssetRequest {
            id: 1,
            name: Some("Updated Bitcoin".to_string()),
            unit_value: Some(60000.0),
        };

        let result = update_asset(AdminAuth, repository, Json(request)).await;
        assert!(result.is_ok());
        if let Ok(Json(Some(asset))) = result {
            assert_eq!(asset.id, 1);
            assert_eq!(asset.name, "Updated Bitcoin");
            assert_eq!(asset.unit_value, 60000.0);

            insta::assert_json_snapshot!(asset);
        }
    }
    
}

