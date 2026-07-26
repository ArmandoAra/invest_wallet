use password_auth::VerifyError;

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
    username: String,
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
}
