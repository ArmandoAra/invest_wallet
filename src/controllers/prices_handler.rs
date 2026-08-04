use axum::{
    extract::State,
    response::sse::{Event, Sse},
};
use crate::app::AppState;
use std::convert::Infallible;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt; // Importante para el .map

pub async fn sse_prices_handler(
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    
    // Nos suscribimos al megáfono
    let rx = state.tx.subscribe();

    // Convertimos el receptor en un Stream que Axum pueda enviar al cliente
    let stream = BroadcastStream::new(rx).filter_map(|msg| {
        // Ignoramos mensajes defectuosos o lag del canal
        let update = msg.ok()?; 
        
        // Convertimos el struct PriceUpdate a un string JSON
        let json_data = serde_json::to_string(&update).ok()?;
        
        // Creamos el evento SSE
        Some(Ok(Event::default()
            .event("price_update") // Nombre del evento
            .data(json_data)))     // La carga útil
    });

    // Devolvemos el flujo continuo
    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("keep-alive-text"),
    )
}