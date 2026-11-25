//La couche réseau (net.rs) : gère la connexion et pousse les événements serveur vers la boucle principale.
//Appel du crate Event
use crate::event::AppEvent;
//Serialisation - Deserialisation
use serde::{Deserialize, Serialize};
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
    event_x: mpsc::UnboundedSender<AppEvent>,
    mut net_rx: mpsc::UnboundedReceiver<WireMessage>,
) {
    tokio::spawn(async move {
        while let Some(msg) = net_rx.recv().await {
            //Serialise les données en JSON + \n + ecrire dans TcpStream / Websocket
            let WireMessage::Chat {
                channel,
                author,
                content,
            } = msg;
            let _ = event_x.send(AppEvent::NetworkMessage {
                channel,
                author: format!("{author} (envoyé)"),
                content,
            });
        }
    });
}
