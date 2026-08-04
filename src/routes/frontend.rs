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
    pub owned_assets: Vec<OwnedAsset>,
    pub available_assets: Vec<Asset>,
    pub user: UserAuth,
    pub current_user: Option<String>,
    pub portfolio_total: f64, 
    pub portfolio_delta: f64, 
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
        .map_err(|_| AppError::DatabaseError(sqlx::Error::RowNotFound))?; 

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
    use axum::response::IntoResponse;
    use axum::http::header::SET_COOKIE;

    // =====================================================================
    // TESTS DE AUTENTICACIÓN
    // =====================================================================

#[sqlx::test]
async fn test_login_creates_user_and_sets_cookie(pool: PgPool) {

    unsafe {
        std::env::set_var("ADMIN_SECRET_KEY", "supersecretkey_to_use_for_a_supersecret");
    }

    let repository = Repository::from(pool.clone());
    let jar = CookieJar::new(); 
    let form = Form(LoginForm {
        username: "newuser".to_string(),
        password: "testpassword".to_string(),
    });

    // Ejecutamos el handler. Si sigue fallando aquí, fíjate en el mensaje que imprimirá.
    let response_result = login(repository, jar, form).await;
    assert!(response_result.is_ok(), "El login falló con el error: {:?}", response_result.err());
    
    let response = response_result.unwrap().into_response();

    // 2. SOLUCIÓN AL CÓDIGO DE ESTADO: 
    // No busques un código específico, comprueba que se comporte como cualquier redirección.
    assert!(
        response.status().is_redirection(), 
        "Se esperaba una redirección, pero devolvió el estado: {}", 
        response.status()
    );

    // 3. SOLUCIÓN A LA CABECERA:
    // Usamos la constante nativa y segura de HTTP
    assert!(
        response.headers().contains_key(SET_COOKIE),
        "El login no devolvió la cabecera Set-Cookie. Cabeceras devueltas: {:?}",
        response.headers()
    );

    // Verificamos la Base de Datos
    let user_exists = sqlx::query!("SELECT id FROM users WHERE username = 'newuser'")
        .fetch_optional(&pool)
        .await
        .unwrap();
    
    assert!(user_exists.is_some(), "El usuario no se guardó en la base de datos");
}

    use axum::http::HeaderMap;

#[tokio::test]
async fn test_logout_removes_cookie() {
    // 1. Simulamos que el navegador envía la cookie en los headers de la petición HTTP
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::COOKIE,
        "token=fake_token_data".parse().unwrap(),
    );
    
    // 2. Construimos el CookieJar a partir de los headers, tal como lo hace Axum al recibir un Request
    let jar = CookieJar::from_headers(&headers);
    
    // 3. Ejecutamos el handler
    let response = logout(jar).await.into_response();

    // 4. Verificamos la redirección
    assert_eq!(response.status(), axum::http::StatusCode::SEE_OTHER);
    
    // 5. Ahora la cabecera Set-Cookie SÍ debe existir, porque el sistema detecta que 
    // está destruyendo una cookie preexistente y necesita avisarle al navegador.
    let cookie_header = response.headers()
        .get("set-cookie")
        .expect("El handler no devolvió la cabecera Set-Cookie")
        .to_str()
        .unwrap();
    
    // 6. Verificamos que se esté borrando (Max-Age=0 o valor vacío/expirado)
    assert!(cookie_header.contains("token="));
    assert!(cookie_header.contains("Max-Age=0") || cookie_header.contains("Expires="));
}
    // =====================================================================
    // TESTS DE CRUD DE PORTAFOLIO (ASSETS)
    // =====================================================================

    #[sqlx::test(fixtures("insert_owned_asset.sql"))]
    async fn test_purchase_asset_handler(pool: PgPool) {
        let repository = Repository::from(pool.clone());
        let user = UserAuth::new(1, "testuser".to_string());
        
        let form = Form(PurchaseAssetForm {
            asset_id: 1,
            unit_value: 50000.0,
            quantity_owned: 0.5,
        });

        // 1. Ejecutamos handler
        let result = purchase_asset(repository, user, form).await;
        assert!(result.is_ok(), "El handler de compra falló");

        // 2. Verificamos BD (Única fuente de la verdad)
        let saved_purchase = sqlx::query!(
            "SELECT quantity_owned, bought_for FROM owned_assets WHERE user_id = 1 AND asset_id = 1"
        )
        .fetch_one(&pool)
        .await
        .expect("No se encontró el activo comprado en la DB");

        assert_eq!(saved_purchase.quantity_owned, 0.5);
        assert_eq!(saved_purchase.bought_for, 50000.0);
    }

    #[sqlx::test(fixtures("update_owned_asset.sql"))]
    async fn test_update_owned_asset_history_handler(pool: PgPool) {
        let repository = Repository::from(pool.clone());
        let user = UserAuth::new(1, "testuser".to_string());
        
        // Simulamos que editamos el historial ID = 1
        let path = Path(1);
        let form = Form(UpdateHistoryForm {
            bought_for: 30000.0,
            quantity_bought: 2.0,
        });

        let result = update_owned_asset_history(repository, user, path, form).await;
        assert!(result.is_ok(), "El handler de actualización falló");

        let updated = sqlx::query!("SELECT bought_for, quantity_owned FROM owned_assets WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(updated.bought_for, 30000.0);
        assert_eq!(updated.quantity_owned, 2.0);
    }

    #[sqlx::test(fixtures("insert_owned_asset.sql"))]
    async fn test_delete_owned_asset_history_handler(pool: PgPool) {
        let repository = Repository::from(pool.clone());
        let user = UserAuth::new(1, "testuser".to_string());
        let path = Path(1); // history_id = 1

        let result = delete_owned_asset_history(repository, user, path).await;
        assert!(result.is_ok());

        let deleted = sqlx::query!("SELECT id FROM owned_assets WHERE id = 1")
            .fetch_optional(&pool)
            .await
            .unwrap();

        assert!(deleted.is_none(), "El historial no se eliminó de la BD");
    }

    #[sqlx::test(fixtures("insert_owned_asset.sql"))]
    async fn test_delete_entire_owned_asset_handler(pool: PgPool) {
        let repository = Repository::from(pool.clone());
        let user = UserAuth::new(1, "testuser".to_string());
        let path = axum::extract::Path(1); // asset_id = 1

        let result = delete_owned_asset(repository, user, path).await;
        assert!(result.is_ok());

        // Debe haber borrado todo lo relacionado a ese asset_id para ese usuario
        let remaining = sqlx::query!(
            "SELECT id FROM owned_assets WHERE user_id = 1 AND asset_id = 1"
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(remaining.len(), 0, "Aún quedan registros del activo en la BD");
    }
}