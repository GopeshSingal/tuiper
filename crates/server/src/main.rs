use protocols::{PlayerResult, ServerMessage};

use axum::extract::State;
use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use std::collections::HashMap;
use tokio::sync::{mpsc, RwLock};

type SharedState = Arc<AppState>;

struct AppState {
    queues: RwLock<HashMap<u32, Vec<u64>>>,
    connection_txs: RwLock<HashMap<u64, mpsc::Sender<ServerMessage>>>,
    races: RwLock<HashMap<String, (u64, u64)>>,
    race_results: RwLock<HashMap<String, (Option<PlayerResult>, Option<PlayerResult>)>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            queues: RwLock::new(HashMap::new()),
            connection_txs: RwLock::new(HashMap::new()),
            races: RwLock::new(HashMap::new()),
            race_results: RwLock::new(HashMap::new()),
        }
    }
}

#[tokio::main]
async fn main() {
    let state: SharedState = Arc::new(AppState::new());
    let app = axum::Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state);
    let addr = std::new::SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
