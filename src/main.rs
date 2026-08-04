mod app;

pub mod auth;
pub mod models;
pub mod routes;
pub mod errors;
pub mod repository;
pub mod workers;
pub mod controllers;

use crate::app::App;

#[tokio::main] //Permite a la funcion main ser asincrona(Inicializa un contexto de ejecucion asincrono)
async fn main() -> color_eyre::Result<()> {
    App::start().await
}
