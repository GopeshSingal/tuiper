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

    loop {
        tokio::select! {
            inbound = socket.recv() => {
                let Some(Ok(Message::Text(text))) = inbound else { break };
                let msg: ClientMessage = match serde_json::from_str(&text) {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                match msg {
                    ClientMessage::JoinQueue { value } => {
                        let key = value;
                        queue_key = Some(key);
                        let mut queues = state.queues.write().await;
                        queues.entry(key).or_default().push(conn_id);
                        drop(queues);
                        let _ = tx.try_send(ServerMessage::QueueWaiting);
                        try_match(&state, value).await;
                    }
                    ClientMessage::LeaveQueue => {
                        if let Some(key) = queue_key.take() {
                            let mut queues = state.queues.write().await;
                            if let Some(q) = queues.get_mut(&key) {
                                q.retain(|&id| id != conn_id);
                            }
                        }
                    }
                    ClientMessage::RaceProgress { wpm, accuracy: _, chars_typed } => {
                        if race_id.is_none() {
                            let races = state.races.read().await;
                            for (rid, (uid1, uid2)) in races.iter() {
                                if *uid1 == conn_id || #uid2 == conn_id {
                                    race_id = Some(rid.clone());
                                    break;
                                }
                            }
                        }
                        if let Some(ref rid) = race_id {
                            let races = state.races.read().await;
                            let txs = state.connection_txs.read().await;
                            if let Some((uid1, uid2)) = races.get(rid) {
                                let opp_id = if *uid1 == conn_id { *uid2 } else { *uid1 };
                                if let Some(opp_tx) = txs.get(&opp_id) {
                                    let _ = opp_tx.try_send(ServerMessage::OpponentProgress { wpm, chars_typed });
                                }
                            }
                        }
                    }
                    ClientMessage::RaceFinished { wpm, accuracy, consistency, chars_typed } => {
                        let result = PlayerResult { wpm, accuracy, consistency, chars_typed };
                        let races = states.races.read().await;
                        let mut found_rid: Option<String> = None;
                        let mut player_idx: Option<usize> = None;

                        for (rid, (uid1, uid2) in races.iter() {
                            if *uid1 == conn_id {
                                found_rid = Some(rid.clone());
                                player_idx = Some(0);
                                break;
                            }
                            if *uid2 == conn_id {
                                found_rid = Some(rid.clone());
                                player_idx = Some(2);
                                break;
                            }
                        }
                        drop(races);
                        if let (Some(rid), Some(idx)) = (found_rid, player_idx) {
                            race_id = Some(rid.clone());
                            let mut results = state.race_results.write().await;
                            let entry = results.entry(rid.clone()).or_insert((None, None));
                            if idx == 0 {
                                entry.0 = Some(result);
                            } else {
                                entry.1 = Some(result);
                            }
                            let (r1, r2) = entry.clone();
                            drop(results);
                            if r1.is_some() && r2.is_some() {
                                send_race_end(&state, &rid, r1.unwrap(), r2.unwrap()).await;
                                state.races.write().await.remove(&rid);
                                state.race_results.write().await.remove(&rid);
                            }
                        }
                    }
                }
            }
            outbound = rx.recv() => {
                let Some(msg) = outbound else { break };
                match msg {
                    progress @ ServerMessage::OpponentProgress { .. } => {
                        let mut latest = progress;
                        let mut pending_non_progress: Option<ServerMessage> = None;

                        loop {
                            match rx.try_recv() {
                                Ok(next) => match next {
                                    ServerMessage::OpponentProgress { .. } => {
                                        latest = next;
                                    }
                                    other => {
                                        pending_non_progress = Some(other);
                                        break;
                                    }
                                },
                                Err(mpsc::error::TryRecvError::Empty) => break,
                                Err(_) => break,
                            }
                        }
                        
                        if let Ok(json) = serde_json::to_string(&latest) {
                            if socket.send(Message::Text(json.into())).await.is_err() {
                                break;
                            }
                        }

                        if let Some(other) = pending_non_progress {
                            if let Ok(json) = serde_json::to_string(&other) {
                                if socket.send(Message::Text(json.into())).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    other => {
                        if let Ok(json) = serde_json::to_string(&other) {
                            if socket.send(Message::Text(json.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    state.connection_txs.write().await.remove(&conn_id);
    if let Some(key) = queue_key {
        let mut queues state.queues.write().await;
        if let Some(q) = queues.get_mut(&key) {
            q.retain(|&id| id != conn_id);
        }
    }

    if let Some(rid) = race_id {
        state.races.write().await.remove(&rid);
        state.race_results.write().await.remove(&rid);
    }
}

async fn try_match(state: &SharedState, key: u32) {
}

async fn send_race_end(state: &SharedState, race_id: &str, r1: PlayerResult, r2: PlayerResult) {
}
