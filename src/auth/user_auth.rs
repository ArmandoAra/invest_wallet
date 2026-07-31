use axum::extract::{FromRequestParts};
use axum_extra::extract::cookie::CookieJar;
use jwt_simple::algorithms::MACLike;
use jwt_simple::claims::Claims;
use jwt_simple::reexports::coarsetime::Duration;
use crate::app::{AppState};
use password_auth::VerifyError;
use crate::errors::AppError;
use jwt_simple::prelude::{HS256Key};

use crate::repository::Repository;
//Utilizamos password-auth


//Estructura para el usuario no autenticado
pub struct UnauthenticatedUser {
    username: String,
    password: String,
}

//Implementando dos usos de la estructura UnauthenticatedUser, uno para crear un nuevo usuario y otro para iniciar sesión
impl UnauthenticatedUser {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    pub async fn authenticate(&self, repository: &Repository) -> Result<UserAuth, crate::errors::AppError> {
        //Obtenemos el usuario de la base de datos
        let user_record = match repository.find_by_username(&self.username).await? {
            Some(user) => user,
            None => return Err(crate::errors::AppError::UserDoesNotExist),
        };

        //Verificamos la contraseña
        match password_auth::verify_password(&self.password, &user_record.password_hash) {
            Ok(_) => Ok(UserAuth::new(user_record.id, user_record.username)),
            Err(VerifyError::PasswordInvalid) => Err(crate::errors::AppError::InvalidCredentials),
            Err(VerifyError::Parse(_)) => panic!("Error parsing password hash"),
        }
    }

    pub async fn register(&self, repository: &Repository) -> Result<UserAuth, crate::errors::AppError> {
    // 1. Validaciones previas indispensables, en caso de alguno de estos errores, debo manejar que no se haga la redireccion y muestre un mensaje de error en la pagina de login, para eso necesitamos que la funcion login_page reciba como parametro un Option<String> que sera el mensaje de error, y que se renderice en la plantilla de login.html
    if self.username.trim().is_empty() || self.password.len() < 2 {
        return Err(crate::errors::AppError::BadRequest("Invalid data".into()));
    }

    // 2. Evaluamos el Option con match
    match repository.find_by_username(&self.username).await? {
        Some(_) => Err(crate::errors::AppError::UserAlreadyExists),
        None => {
            let password_hash = password_auth::generate_hash(&self.password);
            
            // 3. El insert aún debe capturar errores de unicidad de la base de datos
            // para evitar el fallo si el usuario fue creado milisegundos antes.
            let user_record = repository.insert_user_to_db(&self.username, &password_hash).await?;
            
            Ok(UserAuth::new(user_record.id, user_record.username))
        }
    }
}
}

//Estructura para el usuario autenticado, que se va a usar en los endpoints que requieren autenticación de usuario
pub struct UserAuth {
    user_id: i64,
    pub username: String,
}

impl UserAuth {
    pub fn new(user_id: i64, username: String) -> Self {
        Self { user_id, username }
    }

    pub const  fn user_id(&self) -> i64 {
        self.user_id
    }

    pub const fn username(&self) -> &String {
        &self.username
    }

    //generando el token con la libreria jwt-simple 
    pub fn auth_token(self) -> Result<String, AppError> {
        let key = HS256Key::from_bytes(b"my_secret_key");
        let claims = Claims::with_custom_claims(UserClaims::from(self), Duration::from_mins(10));
        let token = key.authenticate(claims)?;
        Ok(token)
    }

    pub fn from_auth_token(token: &str) -> Result<Self, AppError> {
        let key =  HS256Key::from_bytes(b"my_secret_key");
        let claims = key.verify_token::<UserClaims>(token, None)?.custom;
        Ok(UserAuth {
            user_id: claims.user_id,
            username: claims.username,
        })
    }

}

// Extayendo la cookie de sesion del request, para eso implementamos el trait FromRequestParts para UserAuth, y le decimos que el estado de la aplicacion es AppState, y que se inicializa con   AppState::new()
impl FromRequestParts<AppState> for UserAuth {
    type Rejection = crate::errors::AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &AppState,
    ) ->  Result<Self, Self::Rejection>  {
        //Reconstruimos la cookie jar a partir de los headers de la request, y axum se encarga de inyectarla automaticamente, para eso implementamos el trait FromRequestParts para CookieJar, y le decimos que el estado de la aplicacion es AppState, y que se inicializa con   AppState::new()
        let jar = CookieJar::from_headers(&parts.headers);

        let token = jar.get("token").ok_or(crate::errors::AppError::MissingAuthorization)?;

        UserAuth::from_auth_token(token.value()).map_err(|_| crate::errors::AppError::Unauthorized)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct UserClaims {
    user_id: i64,
    username: String,
}

impl From<UserAuth> for UserClaims {
    fn from(UserAuth { user_id, username }: UserAuth ) -> Self {
        Self {
            user_id,
            username
        }
    }
}