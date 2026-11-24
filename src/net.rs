//La couche réseau (net.rs) : gère la connexion et pousse les événements serveur vers la boucle principale.
//Appel du crate Event
use crate::event::AppEvent;
//Serialisation - Deserialisation
use serde::{Deserialize, Serialize};
//Synch - Async
use tokio::sync::mpsc;

//Enumeration du chat -> lien avec AppState
#[derive(Serialize, Deserialize, Debug)]
pub enum WireMessage{
    Chat{
        channel: String,
        author: String,
        content: String,
    },
}

//Simutateur => a remplacer par TcpStream ou tokio-tungstenite
pub fn spawn_network_task(tokio_x: mpsc::UnboundedSender<AppEvent>){
    tokio::spawn(async move {
        //Timer entre chaque envoi de message
        tokio::time::sleep(std::time::Duration::from_secs(2)).await; //2 secondes entre chaque message
        //Faux message de test
        let _ = tokio_x.send(AppEvent::NetworkMessage {
            channel: "general".to_string(),
            author: "Bot de test".to_string(),
            content: "Bonjour je suis un bot de test du chat Rust TUI".to_string(),
        });
    });
}