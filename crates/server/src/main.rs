use protocols::{PlayerResult, ServerMessage};

use axum::extract::State;
use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::{mpsc, RwLock};

type SharedState = Arc<AppState>;

static NEXT_CONN_ID: AtomicU64 = AtomicU64::new(0);

const OUTBOUND_CHANNEL_CAPACITY: usize = 32;

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

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<SharedState>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: SharedState) {
    let (tx, mut rx) = mpsc::channel::<ServerMessage>(OUTBOUND_CHANNEL_CAPACITY);
    let conn_id = NEXT_CONN_ID.fetch_add(1, Ordering::Relaxed);
    state.connection_txs.write().await.insert(conn_id, tx.clone());
    let mut queue_key: Option<u32> = None;
    let mut race_id: Option<String> = None;
}

