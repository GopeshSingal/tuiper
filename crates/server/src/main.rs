use common::{now_unix_ms, MULTIPLAYER_GRACE_PERIOD_SECS};
use protocols::{ClientMessage, PlayerResult, RaceResults, ServerMessage, Winner};

use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use axum::routing::get;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

type SharedState = Arc<AppState>;

static NEXT_CONN_ID: AtomicU64 = AtomicU64::new(0);

const OUTBOUND_CHANNEL_CAPACITY: usize = 256;

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

fn listen_port() -> u16 {
    std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080)
}

#[tokio::main]
async fn main() {
    let port = listen_port();
    let state: SharedState = Arc::new(AppState::new());
    let app = axum::Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state);
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    eprintln!("tuiper-server listening on http://0.0.0.0:{}/ws", port);
    eprintln!("tuiper-server build id: 1");
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
                        let _ = tx.try_send(ServerMessage::Queue);
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
                        let current_rid = {
                            let races = state.races.read().await;
                            races
                                .iter()
                                .find(|(_, (uid1, uid2))| *uid1 == conn_id || *uid2 == conn_id)
                                .map(|(rid, _)| rid.clone())
                        };
                        if let Some(ref rid) = current_rid {
                            let races = state.races.read().await;
                            let txs = state.connection_txs.read().await;
                            if let Some((uid1, uid2)) = races.get(rid) {
                                let opp_id = if *uid1 == conn_id { *uid2 } else { *uid1 };
                                if let Some(opp_tx) = txs.get(&opp_id) {
                                    let _ = opp_tx
                                        .send(ServerMessage::OpponentProgress { wpm, chars_typed })
                                        .await;
                                }
                            }
                        }
                    }
                    ClientMessage::RaceFinished { wpm, accuracy, consistency, chars_typed } => {
                        let result = PlayerResult { wpm, accuracy, consistency, chars_typed };
                        let races = state.races.read().await;
                        let mut found_rid: Option<String> = None;
                        let mut player_idx: Option<usize> = None;

                        for (rid, (uid1, uid2)) in races.iter() {
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
        let mut queues = state.queues.write().await;
        if let Some(q) = queues.get_mut(&key) {
            q.retain(|&id| id != conn_id);
        }
    }

    let mut disconnect_rid: Option<String> = None;
    {
        let races = state.races.read().await;
        for (rid, (uid1, uid2)) in races.iter() {
            if *uid1 == conn_id || *uid2 == conn_id {
                disconnect_rid = Some(rid.clone());
                break;
            }
        }
    }
    if let Some(rid) = disconnect_rid {
        state.races.write().await.remove(&rid);
        state.race_results.write().await.remove(&rid);
    }
}

async fn try_match(state: &SharedState, key: u32) {
    let value = key;
    let mut queues = state.queues.write().await;
    let mut queue = queues.remove(&key).unwrap_or_default();
    if queue.len() < 2 {
        queues.insert(key, queue);
        return;
    }
    let uid1 = queue.remove(0);
    let uid2 = queue.remove(0);
    if !queue.is_empty() {
        queues.insert(key, queue);
    }
    drop(queues);

    let txs = state.connection_txs.read().await;
    let (tx1, tx2) = match (txs.get(&uid1), txs.get(&uid2)) {
        (Some(t1), Some(t2)) => (t1.clone(), t2.clone()),
        _ => return,
    };
    drop(txs);

    let race_id = uuid::Uuid::new_v4().to_string();
    let seed = rand::random::<u64>();
    let start_at_unix_ms = now_unix_ms().saturating_add(MULTIPLAYER_GRACE_PERIOD_SECS.saturating_mul(1000));
    let start1 = ServerMessage::RaceStart {
        race_id: race_id.clone(),
        value,
        seed,
        start_at_unix_ms,
    };
    let start2 = ServerMessage::RaceStart {
        race_id: race_id.clone(),
        value,
        seed,
        start_at_unix_ms,
    };

    state.races.write().await.insert(race_id.clone(), (uid1, uid2));
    state.race_results.write().await.insert(race_id, (None, None));

    let _ = tx1.send(start1).await;
    let _ = tx2.send(start2).await;
}

async fn send_race_end(state: &SharedState, race_id: &str, r1: PlayerResult, r2: PlayerResult) {
    let winner = if r1.wpm > r2.wpm {
        Some(Winner::Me)
    } else {
        Some(Winner::Opponent)
    };
    
    let results1 = RaceResults {
        me: r1.clone(),
        opponent: r2.clone(),
        winner: winner.clone(),
    };
    let results2 = RaceResults {
        me: r2,
        opponent: r1,
        winner: winner.map(|w| match w {
            Winner::Me => Winner::Opponent,
            Winner::Opponent => Winner::Me,
        }),
    };

    let (tx1, tx2) = {
        let races = state.races.read().await;
        let txs = state.connection_txs.read().await;
        let Some((uid1, uid2)) = races.get(race_id) else { return };
        let Some(tx1) = txs.get(uid1) else { return };
        let Some(tx2) = txs.get(uid2) else { return };
        (tx1.clone(), tx2.clone())
    };

    let _ = tx1.send(ServerMessage::RaceEnd { results: results1 }).await;
    let _ = tx2.send(ServerMessage::RaceEnd { results: results2 }).await;
}
