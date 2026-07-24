use crate::{app::AppState, errors::AppError};
use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};

// Aún lo mantengo como constante para que el código funcione ahora,
// pero te insisto: mueve esto a una variable de entorno con std::env::var.
const ADMIN_SECRET_KEY: &str = "your_admin_secret_key";

pub struct AdminAuth;

impl FromRequestParts<AppState> for AdminAuth {
    // 1. Definimos explícitamente el tipo de rechazo con los errores creados en la enum AppError, para que podamos manejar los errores de manera más clara y consistente en toda la aplicación.
    type Rejection = AppError;

    // 2. Usamos async fn en lugar de complicarnos con impl Future
    // 3. Corregimos el tipo de 'parts' a '&mut Parts'
    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if let Some(auth_header) = parts.headers.get(AUTHORIZATION) {
            if let Ok(auth_str) = auth_header.to_str() {
                // Lógica de validación
                if auth_str == format!("Bearer {}", ADMIN_SECRET_KEY) {
                    return Ok(AdminAuth);
                }
            }
        }

        // Si falta el header o no coincide, rechazamos la petición
        Err(AppError::MissingAuthorization)
    }
}
