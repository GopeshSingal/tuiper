use futures_util::{SinkExt, StreamExt};
use std::sync::mpsc;
use tokio::sync::mpsc as tmpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub fn run_ws_thread(
    url: String,
    main_tx: mpsc::Sender<ServerMessage>,
    app_rx: mpsc::Receiver<ClientMessage>,
) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(ws_loop(url, main_tx, app_rx));
    });
}

async fn ws_loop(
    url: String,
    main_tx: mpsc::Sender<ServerMessage>,
    app_rx: mpsc::Receiver<ClientMessage>,
) {
    let (ws_stream, _) = match connect_async(&url).await {
        Ok(x) => x,
        Err(e) => {
            let _ = main_tx.send(ServerMessage::Error {
                message: format!("Connection failed: {}", e),
            });
            return;
        }
    };
    let (mut write, mut read) = ws_stream.split();

    let (cmd_tx, mut cmd_rx) = tmpsc::unbounded_channel::<ClientMessage>();
    std::thread::spawn(move || {
        while let Ok(cmd) = app_rx.recv() {
            let _ = cmd_tx.send(cmd);
        }
    });

    let main_tx2 = main_tx.clone();
    let read_handle = tokio::spawn(async move {
        while let Some(Ok(msg)) = read.next().await {
            if let Message::Text(s) = msg {
                if let Ok(parsed) = serde_json::from_str::<ServerMessage>(&s) {
                    let _ = main_tx2.send(parsed);
                }
            }
        }
    });

    let write_handle = tokio::spawn(async move {
        while let Some(client_msg) = cmd_rx.recv().await {
            if let Ok(json) = serde_json::to_string(&client_msg) {
                if write.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    tokio::select! {
        _ = read_handle => {}
        _ = write_handle => {}
    }
}
