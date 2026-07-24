use std::convert::Infallible;
use crate::models::Asset;
use axum::extract::FromRequestParts;
use sqlx::PgPool;

use crate::app::AppState;

pub struct Repository {
    db: PgPool,
}

//Aqui definimos las funciones que van a interactuar con la base de datos, como por ejemplo crear, leer, actualizar y eliminar assets de la base de datos. Estas funciones van a ser llamadas desde las rutas, y van a recibir como parametro el estado de la aplicacion, para poder acceder a la conexion a la base de datos.
impl Repository {
    pub async fn list_assets_from_db(&self) -> sqlx::Result<Vec<Asset>> {
        //Aqui hacemos la consulta a la base de datos para obtener todos los assets, y los devolvemos como un vector de assets
        let assets = sqlx::query_as!(Asset, "SELECT id, name, unit_value FROM assets;")
            .fetch_all(&self.db)
            .await?;
        Ok(assets)
    }

    pub async fn insert_asset_to_db(&self, name: String, unit_value: f64) -> sqlx::Result<Asset> {
        let asset = sqlx::query_as!(
            Asset,
            "INSERT INTO assets (name, unit_value) VALUES ($1, $2) RETURNING id, name, unit_value;",
            name,
            unit_value
        )
        .fetch_one(&self.db)
        .await?;
        Ok(asset)
    }

    pub async fn update_asset_to_db(&self, id: i64, name: Option<String>, unit_value: Option<f64>) -> sqlx::Result<Option<Asset>> {
        let asset = sqlx::query_as!(
            Asset,
            "UPDATE assets SET name = COALESCE($2, name), unit_value = COALESCE($3, unit_value) WHERE id = $1 RETURNING id, name, unit_value;",
            id,
            name,
            unit_value
        )
        .fetch_optional(&self.db)
        .await?;
        Ok(asset)
    }
}

//haremos que repository sea tambien una dependencia inyectada por el propio axon, esto lo pasamos como argumento a las funciones de las rutas, y axum se encargara de inyectar la dependencia automaticamente, para eso implementamos el trait FromRequestParts para Repository, y le decimos que el estado de la aplicacion es AppState, y que se inicializa con   AppState::new()
impl FromRequestParts<AppState> for Repository {
    type Rejection = Infallible;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self { db: state.db.clone() })
    }
}


#[cfg(test)]
impl From<PgPool> for Repository {
    fn from(db: PgPool) -> Self {
        Self { db }
    }
}