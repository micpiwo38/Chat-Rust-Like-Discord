use std::io;
use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;

// Helper => systeme
fn system_event_json(channel: &str, content: &str) -> String {
    serde_json::json!({
        "Chat": {
            "channel": channel,
            "author": "Système",
            "content": content,
        }
    })
        .to_string()
}

fn extract_author(val: &serde_json::Value) -> Option<String> {
    let author = val
        .get("Chat")
        .and_then(|c| c.get("author"))
        .and_then(|a| a.as_str())?
        .trim();

    if author.is_empty() || author == "Système" {
        None
    } else {
        Some(author.to_string())
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let addr = "0.0.0.0:4000";
    let listener = TcpListener::bind(addr).await?;
    println!("Serveur TCP en écoute sur : {}", addr);

    // Canal de diffusion partagé entre tous les clients connectés
    let (tx, _rx) = broadcast::channel::<String>(256);

    loop {
        let (socket, peer_addr) = listener.accept().await?;
        println!(
            "Un nouveau client c'est connecté a l'adresse : {} partagée !",
            peer_addr
        );

        let join_msg = system_event_json(
            "general",
            &format!("Un client a rejoint le serveur ({peer_addr})"),
        );
        let _ = tx.send(join_msg);

        let client_tx = tx.clone();
        let client_rx = tx.subscribe();

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, peer_addr, client_tx, client_rx).await {
                eprintln!("Erreur => connexion terminée !  {}", e);
            }
        });
    }
}

async fn handle_client(
    socket: TcpStream,
    peer_addr: SocketAddr,
    tx: broadcast::Sender<String>,
    mut rx: broadcast::Receiver<String>,
) -> io::Result<()> {
    let (reader, mut writer) = socket.into_split();
    let mut lines = BufReader::new(reader).lines();
    let mut last_author: Option<String> = None;

    loop {
        tokio::select! {
            // 1. Message entrant depuis les clients TCP
            line_res = lines.next_line() => {
                match line_res {
                    Ok(Some(line)) => {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        // Vérification que la ligne est du JSON valide
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                            if let Some(author) = extract_author(&val) {
                                last_author = Some(author);
                            }
                            let _ = tx.send(trimmed.to_string());
                        }
                    }
                    // Client déconnecté proprement (EOF) ou fermeture brutale (ex. 10054)
                    Ok(None) | Err(_) => break,
                }
            }
            // 2. Message diffusé par le serveur vers tous les clients connectés
            broadcast_res = rx.recv() => {
                match broadcast_res {
                    Ok(msg) => {
                        if writer.write_all(msg.as_bytes()).await.is_err() {
                            break;
                        }
                        if writer.write_all(b"\n").await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        eprintln!("Le client est en retard : {} le message va etre ignoré !", skipped);
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }

    let leave_text = match &last_author {
        Some(pseudo) => format!("{pseudo} s'est déconnecté"),
        None => format!("Un client s'est déconnecté ({peer_addr})"),
    };
    let leave_msg = system_event_json("general", &leave_text);
    let _ = tx.send(leave_msg);

    Ok(())
}