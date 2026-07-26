use axum::{Router, response::Html, routing::get, extract::Form};
use askama::Template;


use crate::app::AppState;
use crate::auth::user_auth::UnauthenticatedUser;
use crate::errors::AppError;
use crate::repository::Repository;

pub fn frontend_routes() -> Router<AppState> {
    Router::new().route("/login", get(login_page).post(login))
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginPage{
    error_message: Option<String>,
}
async fn login_page() -> Result<Html<String>,AppError> {
    let page = LoginPage { error_message: None }.render().map_err(AppError::Template)?;
    Ok(Html(page))
}
//Necesitamos del resultado del formularion
#[derive(serde::Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

//Necesitamos hacer la injeccion de dependencias de Repository en la ruta de login, para eso necesitamos que la funcion login_page reciba como parametro un Repository, y axum se encargara de inyectar la dependencia automaticamente, para eso implementamos el trait FromRequestParts para Repository, y le decimos que el estado de la aplicacion es AppState, y que se inicializa con   AppState::new()
async fn login(repository: Repository, Form(request): Form<LoginForm>) -> Result<Html<String>,AppError> {
   let unauth_user = UnauthenticatedUser::new(request.username, request.password);

    let user = match unauth_user.authenticate(&repository).await {
        Ok(user) => user,
        Err(AppError::UserDoesNotExist) => unauth_user.register(&repository).await?,
        Err(outher_error) => return Err(outher_error),
    };
    
    Ok(Html(format!("Welcome, {}!", user.username())))
}