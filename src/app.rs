use crate::controllers::prices_handler::sse_prices_handler;
use crate::models::PriceUpdate;
use crate::{routes, workers::price_worker};
use axum::{Router, routing::get};
use dotenvy::dotenv;
use sqlx::PgPool;
use tokio::{net::TcpListener, sync::broadcast};
use tracing::info;
use tracing_subscriber::{
    Layer, fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt,
};

// Colocar aqui lo que yo  quiera compartir entre rutas, como por ejemplo la conexion a la base de datos, o el estado de la aplicacion AppState , Necesitamos que sea el mismo vector de assets para todas las rutas, por eso lo ponemos en el estado de la aplicacion
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool, //Esto ayuda a que todas las rutas tengan acceso a la misma conexion a la base de datos, y no tengamos que crear una nueva conexion cada vez que se hace una peticion
    pub tx: broadcast::Sender<PriceUpdate>, // El canal de broadcast
}

impl AppState {
    async fn new() -> color_eyre::Result<Self> {
        //Recibimos la ruta de la db de una variable de entorno.
        dotenv()?;
        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        //Abrimos la conexion a la base de datos, y la guardamos en el estado de la aplicacion, para que todas las rutas tengan acceso a la misma conexion
        let db_connection = PgPool::connect(&db_url).await?;

        //Arrancando el worker de actualizacion de precios
        // Creamos un canal con capacidad para 100 mensajes en cola
        let (tx, _rx) = broadcast::channel(100);
        price_worker::spawn_price_updater(db_connection.clone(), tx.clone());

        Ok(AppState {
            db: db_connection,
            tx,
        })
    }
}

pub struct App;

impl App {
    pub async fn start() -> color_eyre::Result<()> {
        let layer = tracing_subscriber::fmt::layer()
            .with_span_events(FmtSpan::NEW)
            .boxed();
        tracing_subscriber::registry().with(layer).init();

        let state = AppState::new().await?;

        let listener = TcpListener::bind("0.0.0.0:3000").await?;
        let router = Router::new()
            .route("/api/sse/prices", get(sse_prices_handler))
            .nest("/api", routes::api::router())
            .merge(routes::frontend::frontend_routes())
            .with_state(state); //Diciendole que el estado de la aplicacion es AppState, y que se inicializa con   AppState::new()

        info!("Server running");

        // Funcion principal de entrada de axum
        axum::serve(listener, router).await?;
        Ok(())
    }
}
