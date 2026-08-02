use crate::auth::user_auth::UserAuth;
use crate::models::{Asset, OwnedAsset};
use askama::Template;
use axum::response::{IntoResponse, Redirect};
use axum::{Router, extract::{Form, Path}, response::Html, routing::get};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use crate::repository::Repository;

use crate::app::AppState;
use crate::auth::user_auth::UnauthenticatedUser;
use crate::errors::AppError;
use serde::Deserialize;

pub fn frontend_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/login", get(login_page).post(login))
        .route("/logout", get(logout))
        .route("/assets", get(assets).post(purchase_asset))
        .route("/assets/purchase/update/{history_id}", axum::routing::post(update_owned_asset_history))
        .route(
            "/assets/purchase/delete/{asset_id}",
            axum::routing::post(delete_owned_asset_history),
        )
        .route(
            "/assets/delete/{asset_id}",
            axum::routing::post(delete_owned_asset),
        )
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginPage {
    error_message: Option<String>,
    pub current_user: Option<String>,
}

async fn login_page() -> Result<Html<String>, AppError> {
    let page = LoginPage {
        error_message: None,
        current_user: None,
    }
    .render()
    .map_err(AppError::Template)?;
    Ok(Html(page))
}
//Necesitamos del resultado del formularion
#[derive(serde::Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

//Necesitamos hacer la injeccion de dependencias de Repository en la ruta de login, para eso necesitamos que la funcion login_page reciba como parametro un Repository, y axum se encargara de inyectar la dependencia automaticamente, para eso implementamos el trait FromRequestParts para Repository, y le decimos que el estado de la aplicacion es AppState, y que se inicializa con   AppState::new()
async fn login(
    repository: Repository,
    jar: CookieJar, // Extraemos la cookie jar directo del request, para poder almacenar la cookie de sesion en el navegador del usuario. Esta es construida a partir de los headers de la request, y axum se encarga de inyectarla automaticamente, para eso implementamos el trait FromRequestParts para CookieJar, y le decimos que el estado de la aplicacion es AppState, y que se inicializa con   AppState::new()
    Form(request): Form<LoginForm>,
) -> Result<impl IntoResponse, AppError, > {

    let unauth_user = UnauthenticatedUser::new(request.username, request.password);
  
    let user = match unauth_user.authenticate(&repository).await {
        Ok(user) => user,
        Err(AppError::UserDoesNotExist) => unauth_user.register(&repository).await?,
        Err(outher_error) => return Err(outher_error),
    };

    let token = user.auth_token()?;

    //Construimos la cookie de sesion , para pasarla despues al index
    let cookie = Cookie::build(("token", token)).http_only(true);

    // Ok(Html(format!("Welcome, {}!", user.username())))
    // Redirigimos al usuario a la página de inicio después de iniciar sesión o registrarse, Forzando al navegador a almacenar la cookie de sesión en el navegador, para eso necesitamos que la funcion login_page reciba como parametro un Repository, y axum se encargara de inyectar la dependencia automaticamente, para eso implementamos el trait FromRequestParts para Repository, y le decimos que el estado de la aplicacion es AppState, y que se inicializa con   AppState::new()
    Ok((jar.add(cookie), Redirect::to("/"))) // Tambien pasamos las cookies al index
}

async fn logout(jar: CookieJar) -> impl IntoResponse {
    let jar = jar.remove("token");
    (jar, Redirect::to("/"))
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexPage {
    pub current_user: Option<String>,
}

pub async fn index(jar: CookieJar) -> Result<impl IntoResponse, AppError> {
    // Si ya está logueado, lo mandamos directo al dashboard (Assets)
    if jar.get("token").is_some() {
        return Ok(Redirect::to("/assets").into_response());
    }

    // Si NO está logueado, le mostramos la nueva Landing Page
    let page = IndexPage {
        current_user: None, // Es None porque sabemos que no está logueado
    }
    .render()
    .map_err(AppError::Template)?;

    Ok(Html(page).into_response())
}

#[derive(Template)]
#[template(path = "assets.html")]
pub struct AssetsPage {
    owned_assets: Vec<OwnedAsset>,
    available_assets: Vec<Asset>,
    user: UserAuth,
    pub current_user: Option<String>,
    portfolio_total: f64, 
    portfolio_delta: f64, 
}

pub async fn assets(repository: Repository, user: UserAuth) -> Result<Html<String>, AppError> {
    //Usamos tokio::try_join! para ejecutar las dos consultas a la base de datos en paralelo, y asi optimizar el tiempo de respuesta, ya que no necesitamos que una consulta termine para empezar la otra, y axum se encargara de inyectar la dependencia automaticamente, para eso implementamos el trait FromRequestParts para Repository, y le decimos que el estado de la aplicacion es AppState, y que se inicializa con   AppState::new()
    let (owned_assets, available_assets) = tokio::try_join!(
        repository.list_owned_assets_from_db(user.user_id()),
        repository.list_assets_from_db()
    )?;

    let user_name = user.username().clone();

    let portfolio_total: f64 = owned_assets
        .iter()
        .map(|a| a.quantity_owned * a.unit_value)
        .sum();

    // Delta = suma de todas las ganancias/pérdidas
    let portfolio_delta: f64 = owned_assets
        .iter()
        .map(|a| a.value_delta)
        .sum();

    // Renderizamos manualmente a un String y lo envolvemos en Html, de haber un error, redireccionamos a home
    let page = AssetsPage {
        owned_assets,
        available_assets,
        user,
        current_user: Some(user_name),
        portfolio_total,
        portfolio_delta,
    }
    .render()
    .map_err(AppError::Template)?;

    Ok(Html(page))
}

#[derive(Deserialize)]
pub struct PurchaseAssetForm {
    asset_id: i64,
    unit_value: f64,
    quantity_owned: f64,
}

pub async fn purchase_asset(
    repository: Repository,
    user: UserAuth,
    Form(form): Form<PurchaseAssetForm>,
) -> Result<Redirect, AppError> {
    repository
        .insert_owned_asset_to_db(
            user.user_id(),
            form.asset_id,
            form.quantity_owned,
            form.unit_value,
        )
        .await?;

    Ok(Redirect::to("/assets"))
}

#[derive(Deserialize)]
pub struct UpdateHistoryForm {
    pub bought_for: f64,
    pub quantity_bought: f64, 
}

pub async fn update_owned_asset_history(
    repository: Repository,
    user: UserAuth,
    // Atrapamos el ID que viene en la URL
    Path(history_id): Path<i64>, 
    // Atrapamos SOLO los dos inputs que vienen en el formulario
    Form(form): Form<UpdateHistoryForm>,
) -> Result<Redirect, AppError> {
    
    repository
        .update_owned_asset_history_in_db(
            user.user_id(),
            history_id,
            form.quantity_bought,
            form.bought_for,
        )
        .await?;

    Ok(Redirect::to("/assets"))
}

pub async fn delete_owned_asset(
    repository: Repository,
    user: UserAuth,
    axum::extract::Path(asset_id): axum::extract::Path<i64>,
 ) -> Result<Redirect, AppError> {
    repository
        .delete_owned_asset_from_db(user.user_id(), asset_id)
        .await
        .map_err(|_| AppError::DatabaseError(sqlx::Error::RowNotFound))?; // Manejar el error adecuadamente en un caso real

    Ok(Redirect::to("/assets"))
}

pub async fn delete_owned_asset_history(
    repository: Repository,
    user: UserAuth,
    Path(history_id): Path<i64>,
 ) -> Result<Redirect, AppError> {
    repository
        .delete_owned_asset_history_from_db(user.user_id(), history_id)
        .await
        .map_err(|_| AppError::DatabaseError(sqlx::Error::RowNotFound))?; // Manejar el error adecuadamente en un caso real

    Ok(Redirect::to("/assets"))
}
pub mod filters {
    use askama;
    use time::{
        OffsetDateTime, format_description::StaticFormatDescription, macros::format_description,
    };

    #[askama::filter_fn]
    pub fn format_human_readable_datetime(
        datetime: &OffsetDateTime,
        _env: &dyn askama::Values,
    ) -> askama::Result<String> {
        const HUMAN_READABLE_FORMAT : StaticFormatDescription =
            format_description!(version = 2, "[year]-[month]-[day] [hour]:[minute]:[second]");
        datetime.format(&HUMAN_READABLE_FORMAT).map_err(|e| askama::Error::custom(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn create_user_and_authenticate(pool: PgPool)  {
        let repository = Repository::from(pool);
        //Esto va a comprobar que el usuario se puede crear y autenticar correctamente, y que se puede obtener el token de autenticacion el retorno de insert_user_to_db es Result<UserRecord, crate::errors::AppError>
        let unauth_user = UnauthenticatedUser::new("testuser".to_string(), "testpassword".to_string());
        match unauth_user.authenticate(&repository).await {
            Ok(user) => user,
            Err(AppError::UserDoesNotExist) => unauth_user.register(&repository).await.unwrap(),
            Err(outher_error) => panic!("Unexpected error: {:?}", outher_error),
        };
       
    }
}