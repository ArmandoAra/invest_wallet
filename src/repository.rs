use crate::models::{Asset, OwnedAsset, UserRecord};
use axum::extract::FromRequestParts;
use sqlx::PgPool;
use std::convert::Infallible;

use crate::app::AppState;

pub struct Repository {
    db: PgPool,
}

//Aqui definimos las funciones que van a interactuar con la base de datos, como por ejemplo crear, leer, actualizar y eliminar assets de la base de datos. Estas funciones van a ser llamadas desde las rutas, y van a recibir como parametro el estado de la aplicacion, para poder acceder a la conexion a la base de datos.
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

    pub async fn update_asset_to_db(
        &self,
        id: i64,
        name: Option<String>,
        unit_value: Option<f64>,
    ) -> sqlx::Result<Option<Asset>> {
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
impl From<PgPool> for Repository {
    fn from(db: PgPool) -> Self {
        Self { db }
    }
}
