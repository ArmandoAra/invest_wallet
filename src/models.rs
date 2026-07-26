use serde::{Serialize};

#[derive(Serialize, Clone,Debug)]
pub struct Asset {
    pub id: i64,
    pub name: String,
    pub unit_value: f64,
}

//Modelo para la tabla users, que tiene id, username y password_hash
#[derive(Clone,Debug)]
pub struct UserRecord {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
}