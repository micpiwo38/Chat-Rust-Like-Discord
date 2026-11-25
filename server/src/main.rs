use std::io;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> io::Result<()> {
    let addr = "127.0.0.1:4000";
    let listener = TcpListener::bind(addr).await?;
    println!("Serveur TCP en écoute sur : {}", addr);

    //Canal de diffusion partagé entre tous les clients connectés
    let (tx, _rx) = broadcast::channel::<String>(256);

    loop {
        let (socket, peer_addr) = listener.accept().await?;
        println!("Un nouveau client c'est connecté a l'adresse : {} partagée !", peer_addr);

        let client_tx = tx.clone();
        let client_rx = tx.subscribe();

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, client_tx, client_rx).await {
                eprintln!("Erreur => connexion terminée !  {}", e);
            }
        });
    }
}

async fn handle_client(
    socket: TcpStream,
    tx: broadcast::Sender<String>,
    mut rx: broadcast::Receiver<String>,
) -> io::Result<()> {
    let (reader, mut writer) = socket.into_split();
    let mut lines = BufReader::new(reader).lines();

    loop {
        tokio::select! {
            //1. Message entrant depuis les clients TCP
            line_res = lines.next_line() => {
                match line_res?{
                    Some(line) => {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        //Verification que la ligne est du Json valide
                        if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
                            //Ignore l'erreur si un seul client
                            let _ = tx.send(trimmed.to_string());
                        }
                    }
                    //Client déconnecté proprement = EndOfFrame = EOF
                    None => break,
                }
            }
            //2. Message diffusé par le serveur vers tous les clients connectés
            broadcaset_res = rx.recv() => {
                match broadcaset_res{
                    Ok(msg) => {
                        writer.write_all(msg.as_bytes()).await?;
                        writer.write_all(b"\n").await?; //Retour chariot
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        eprintln!("Le client est en retard : {} le message va etre ignoré !", skipped);
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }
    Ok(())
}
