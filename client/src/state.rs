//L'état de l'application (state.rs) : salons, messages, buffer de saisie, zone active (Input ou Channels), scroll.
//Carte de Hachage => résistance contre les attaques de HashDoS
use std::collections::HashMap;


//Login
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenMode{
    Login, Chat
}
//Enumeration des panneau actif
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivePane{
    Channels, Input,
}
//La structure des messages
#[derive(Debug, Clone)]
pub struct ChatMessage{
    pub author: String,
    pub content: String,
}
//Etat generale de l'application
pub struct AppState{
    pub username: String, //Nom utilisateur
    pub channels: Vec<String>, //Tableau des salons
    pub current_channel: usize, //Salon actif
    pub messages: HashMap<String, Vec<ChatMessage>>, //Table de Hash des messages
    pub input: String, //Entrée clavier utilisateur
    pub active_pane: ActivePane, //Panneau actif
    pub should_quit: bool, //Quitter le salon
    pub screen: ScreenMode, //Login - ou le chat
    pub login_input: String,
}

//Implementation de l'etat de l'application
impl AppState{
    //Contructeur a l'instance de l'etat
    pub fn new(username: String) -> Self{
        //Liste de salon exemple
        let channels = vec!["general".to_string(), "video games".to_string(), "test".to_string()];
        //Table de hachage des messages
        let mut messages = HashMap::new();
        //Boucle de parcours des messages
        for channel in &channels{
            //Inserer les messages dans la table de Hash
            messages.insert(channel.clone(), Vec::new());
        }
        Self{
            username,
            channels,
            current_channel: 0,
            messages,
            input: String::new(),
            active_pane: ActivePane::Input, //Enum Input quand dans le salon
            should_quit: false,
            screen: ScreenMode::Login,
            login_input: String::new(),
        }
    }

    //Activer - Desactiver un salon
    pub fn active_channel_name(&self) -> &str{
        &self.channels[self.current_channel]
    }

    //Valider et ajouter un message (Salon + auteur + contenu du message)
    pub fn push_message(&mut self, channel: &str, author: String, content: String){
        self.messages
            .entry(channel.to_string())
            .or_default()
            .push(ChatMessage{author, content});
    }
}
