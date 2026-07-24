use axum::response::IntoResponse;
use axum::http::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Missing authorization")]
    MissingAuthorization,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Assets does not exist")]
    AssetsDoesNotExist,
}

//Ahora definimos como los errores se van a mostrar en la respuesta de la API, para eso implementamos la trait IntoResponse de axum, que nos permite convertir nuestro error en una respuesta HTTP
#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl IntoResponse for AppError {
    //para cada tipo de error, definimos el código de estado HTTP que queremos devolver, y el mensaje de error que queremos mostrar en la respuesta
    fn into_response(self) -> axum::response::Response {
        let error_response = ErrorResponse {
            error: self.to_string(),
        };
        
        let status = match self {
            Self::MissingAuthorization => StatusCode::UNAUTHORIZED,
            Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::AssetsDoesNotExist => StatusCode::NOT_FOUND,
        };

        (status, axum::Json(error_response)).into_response()
    }
}