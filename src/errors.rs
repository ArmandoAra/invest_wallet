use axum::response::{IntoResponse, Redirect}; // <-- Asegúrate de importar Redirect
use axum::http::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Token creation error")]
    TokenCreationError,
    #[error("Missing authorization")]
    MissingAuthorization,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("El usuario ya existe")]
    UserAlreadyExists,
    #[error("User does not exist")]
    UserDoesNotExist,
    #[error("Assets does not exist")]
    AssetsDoesNotExist,
    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),
    #[error(transparent)]
    Template(#[from] askama::Error),
    #[error(transparent)]
    Jwt(#[from] jwt_simple::Error),
}

#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        // 1. Interceptamos los errores de sesión para que hagan una redirección HTTP clásica
        match self {
            Self::MissingAuthorization | Self::Unauthorized => {
                // Al devolver un Redirect, el navegador cambia de página inmediatamente
                return Redirect::to("/login").into_response();
            }
            _ => {} // Si no es un error de autorización, dejamos que siga al paso 2
        };

        // 2. Todo lo que NO sea un problema de sesión, se devuelve como JSON
        let error_response = ErrorResponse {
            error: self.to_string(),
        };
        
        let status = match self {
            Self::TokenCreationError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InvalidCredentials => StatusCode::UNAUTHORIZED, // Quizás este lo quieras dejar como JSON para mostrar un mensaje en el modal de login
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::UserAlreadyExists => StatusCode::CONFLICT,
            Self::UserDoesNotExist => StatusCode::NOT_FOUND,
            Self::AssetsDoesNotExist => StatusCode::NOT_FOUND,
            Self::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Template(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Jwt(_) => StatusCode::INTERNAL_SERVER_ERROR,
            // Los casos MissingAuthorization y Unauthorized ya no llegarán aquí, 
            // pero Rust te exige cubrir todos los casos del enum, así que ponemos un fallback _
            _ => StatusCode::INTERNAL_SERVER_ERROR, 
        };

        (status, axum::Json(error_response)).into_response()
    }
}