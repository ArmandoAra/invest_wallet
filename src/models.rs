use serde::{Serialize, Deserialize};
use time::OffsetDateTime;
use sqlx::types::Json;

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

#[derive(Serialize, Deserialize)]
pub struct PurchaseHistory {
    pub id: i64,
    #[serde(with = "time::serde::iso8601")]
    pub bought_at: OffsetDateTime,
    pub bought_for: f64,
    pub quantity_bought: f64,
    pub value_delta: f64,
}

#[derive(Serialize)]
pub struct OwnedAsset {
    pub id: i64,
    pub name: String,
    pub unit_value: f64,
    pub value_delta: f64, //Cantidad de lucros o perdidas que se han tenido con el asset, calculado como (current_value - bought_for) * quantity_owned
    pub quantity_owned: f64,
    pub purchase_history: Json<Vec<PurchaseHistory>>,
}

#[derive(Deserialize)]
pub struct PurchaseAssetRequest {
    pub quantity_owned: f64,
    pub unit_value: f64,
    pub asset_id: i64,
    pub history_id: i64, // Agregamos un campo opcional para el history_id, que será usado para actualizar una compra específicas
}