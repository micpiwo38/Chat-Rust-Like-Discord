//La couche réseau (net.rs) : gère la connexion et pousse les événements serveur vers la boucle principale.
//Appel du crate Event
use crate::event::AppEvent;
//Serialisation - Deserialisation
use serde::{Deserialize, Serialize};
//Communication async avec le serveur
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
//Protocole de communication et diffusion
use tokio::net::TcpStream;
//Synch - Async
use tokio::sync::mpsc;

//Enumeration du chat -> lien avec AppState
#[derive(Serialize, Deserialize, Debug)]
pub enum WireMessage {
    Chat {
        channel: String,
        author: String,
        content: String,
    },
}

//Simutateur => a remplacer par TcpStream ou tokio-tungstenite
pub fn spawn_network_task(
    addr: String,
    event_tx: mpsc::UnboundedSender<AppEvent>,
    mut net_rx: mpsc::UnboundedReceiver<WireMessage>,
) {
    tokio::spawn(async move {
        let stream = match TcpStream::connect(&addr).await {
            Ok(s) => {
                let _ = event_tx.send(AppEvent::NetworkMessage {
                    channel: "Genral".to_string(),
                    author: "Système".to_string(),
                    content: format!("Bienvenue, vous êtes connecté a : {}!", addr),
                });
                s
            }
            Err(e) => {
                let _ = event_tx.send(AppEvent::NetworkMessage {
                    channel: "General".to_string(),
                    author: "Système".to_string(),
                    content: format!("Erreur de connexion au serveur ! {e}"),
                });
                return;
            }
        };

        //Lecture ecriture
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        //Boucle infinie du chat message => coté client
        loop {
            tokio::select! {
                //1. Reception d'une ligne depuis le serveur TCP
                line_res = lines.next_line() => {
                    match line_res{
                        Ok(Some(line)) => {
                            if let Ok(WireMessage::Chat {channel, author, content}) =
                                serde_json::from_str::<WireMessage>(&line) {
                                if event_tx
                                .send(AppEvent::NetworkMessage {
                                    channel, author, content,
                                }).is_err(){
                                    break;
                                }
                            }
                        }
                        Ok(None) => {
                            let _ = event_tx.send(AppEvent::NetworkMessage {
                                channel: "General".to_string(),
                                author: "Système".to_string(),
                                content: "Vous êtes déconnecté du serveur".to_string(),
                            });
                            break;
                        }
                        Err(e) => {
                            let _ = event_tx.send(AppEvent::NetworkMessage {
                                channel: "General".to_string(),
                                author: "Système".to_string(),
                                content: format!("Erreur de connexion au serveur ! {e}").to_string(),
                            });
                            break;
                        }
                    }
                }
                //2. envoi du message saisis des clients vers le serveur principale
                maybe_msg = net_rx.recv() => {
                    match maybe_msg{
                        Some(msg) => {
                            if let Ok(mut json) = serde_json::to_string(&msg) {
                                json.push('\n');
                                if writer.write_all(json.as_bytes()).await.is_err(){
                                    break;
                                }
                            }
                        }
                        None => break,
                    }
                }
            }
        }
    });
}
