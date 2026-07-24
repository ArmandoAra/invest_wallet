use crate::{models::Asset, routes};
use axum::Router;
use std::{collections::HashMap, sync::Arc};
use tokio::{net::TcpListener, sync::Mutex};
use tracing::info;
use tracing_subscriber::{
    Layer, fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt,
};

// Colocar aqui lo que yo  quiera compartir entre rutas, como por ejemplo la conexion a la base de datos, o el estado de la aplicacion AppState , Necesitamos que sea el mismo vector de assets para todas las rutas, por eso lo ponemos en el estado de la aplicacion
#[derive(Clone)]
pub struct AppState {
    pub assets: Arc<Mutex<HashMap<i64, Asset>>>, // Esto es un hashmap de assets compartido entre todas las rutas, y protegido por un Mutex para que no haya problemas de concurrencia
}

impl AppState {
    fn new() -> Self {
        AppState {
            assets: Default::default(), // Inicializa el hashmap de assets como un hashmap vacio
        }
    }
}

pub struct App;

impl App {
    pub async fn start() -> color_eyre::Result<()> {
        let layer = tracing_subscriber::fmt::layer()
            .with_span_events(FmtSpan::NEW)
            .boxed();
        tracing_subscriber::registry().with(layer).init();

        let listener = TcpListener::bind("0.0.0.0:3000").await?;
        let router = Router::new()
            .nest("/api", routes::api::router())
            .with_state(AppState::new()); //Diciendole que el estado de la aplicacion es AppState, y que se inicializa con   AppState::new()

        info!("Server running");

        // Funcion principal de entrada de axum
        axum::serve(listener, router).await?;
        Ok(())
    }
}
