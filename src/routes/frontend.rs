use askama::Template;
use axum::response::{IntoResponse, Redirect};
use axum_extra::extract::cookie::Cookie;
use axum::{Router, extract::Form, response::Html, routing::get};
use axum_extra::extract::CookieJar;
use crate::auth::user_auth::UserAuth;

use crate::app::AppState;
use crate::auth::user_auth::UnauthenticatedUser;
use crate::errors::AppError;
use crate::repository::Repository;

pub fn frontend_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/login", get(login_page).post(login))
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginPage {
    error_message: Option<String>,
}
async fn login_page() -> Result<Html<String>, AppError> {
    let page = LoginPage {
        error_message: None,
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
    jar: CookieJar,// Extraemos la cookie jar directo del request, para poder almacenar la cookie de sesion en el navegador del usuario. Esta es construida a partir de los headers de la request, y axum se encarga de inyectarla automaticamente, para eso implementamos el trait FromRequestParts para CookieJar, y le decimos que el estado de la aplicacion es AppState, y que se inicializa con   AppState::new()
    Form(request): Form<LoginForm>,
) -> Result<impl IntoResponse, AppError> {
    let unauth_user = UnauthenticatedUser::new(request.username, request.password);

    let user = match unauth_user.authenticate(&repository).await {
        Ok(user) => user,
        Err(AppError::UserDoesNotExist) => unauth_user.register(&repository).await?,
        Err(outher_error) => return Err(outher_error),
    };

    let token = user.auth_token()?;

    //Construimos la cookie de sesion , para pasarla despues al index
    let cookie = Cookie::build(("token", token))
        .http_only(true);

    // Ok(Html(format!("Welcome, {}!", user.username())))
    // Redirigimos al usuario a la página de inicio después de iniciar sesión o registrarse, Forzando al navegador a almacenar la cookie de sesión en el navegador, para eso necesitamos que la funcion login_page reciba como parametro un Repository, y axum se encargara de inyectar la dependencia automaticamente, para eso implementamos el trait FromRequestParts para Repository, y le decimos que el estado de la aplicacion es AppState, y que se inicializa con   AppState::new()
    Ok((jar.add(cookie), Redirect::to("/")))// Tambien pasamos las cookies al index
}   

async fn index(user: UserAuth) -> Result<Html<String>, AppError> {
    Ok(Html(format!("<h1>Welcome {} to the index page!</h1>", user.username())))
}
