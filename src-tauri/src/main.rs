// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager as _;

use std::{
    collections::HashMap,
    env,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::Message;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

#[derive(Debug, thiserror::Error)]
enum Error {
  #[error(transparent)]
  Io(#[from] std::io::Error)
}

// we must manually implement serde::Serialize
impl serde::Serialize for Error {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::ser::Serializer,
  {
    serializer.serialize_str(self.to_string().as_ref())
  }
}

async fn handle_connection<'a>(app_handle:tauri::AppHandle, raw_stream: TcpStream, addr: SocketAddr) {
    let peer_map:tauri::State<PeerMap> = app_handle.state();
    println!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    // Insert the write part of this peer to the peer map.
    let (tx, rx) = unbounded();
    peer_map.lock().unwrap().insert(addr, tx.clone());

    let (outgoing, incoming) = ws_stream.split();

    let broadcast_incoming = incoming.try_for_each(|msg| {
        let text = msg.to_text().unwrap();
        println!("Received a message from {}: {}", addr, text);
        let peers = peer_map.lock().unwrap();

        // We want to broadcast the message to everyone except ourselves.
        let broadcast_recipients =
            peers.iter().filter(|(peer_addr, _)| peer_addr != &&addr).map(|(_, ws_sink)| ws_sink);

        for recp in broadcast_recipients {
            recp.unbounded_send(msg.clone()).unwrap();
        }

        //sending response
        let response = format!("Rust received \"{}\"", text);
        tx.unbounded_send(response.as_str().into()).unwrap();

        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;

    println!("{} disconnected", &addr);
    peer_map.lock().unwrap().remove(&addr);
}


// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
async fn listen<'a>(app_handle:tauri::AppHandle, listener:tauri::State<'_, TcpListener>) -> Result<(), Error>{
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(app_handle.clone(), stream, addr));
    }

    Ok(())
}

#[tokio::main]
pub async fn main() {
    let server = TcpListener::bind("127.0.0.1:9001").await.unwrap();
    let state = PeerMap::new(Mutex::new(HashMap::new()));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(server)
        .manage(state)
        .invoke_handler(tauri::generate_handler![listen])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
