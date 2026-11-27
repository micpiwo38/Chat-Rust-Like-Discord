//L'état de l'application (state.rs) : salons, messages, buffer de saisie, zone active (Input ou Channels), scroll.
//Carte de Hachage => résistance contre les attaques de HashDoS
use std::collections::{HashMap};
use ratatui::layout::Rect;

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
//La structure des messages dans l'input
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
    pub login_input: String, //Champ input connexion
    pub scroll_offset: u16, //0 = collé en bas
    pub channels_area: Rect, //Zone panneau gauche => les salons
    pub input_area: Rect, //Zone panneau droite => chat
    pub users: Vec<(String, bool)>, // Nom utilisateur + status (est en ligne ou non)
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
            username, //Nom utilisateur
            channels, //Liste des salons
            current_channel: 0, //Salon actif
            messages, //Table de Hash des messages
            input: String::new(), //Entree utilisateur
            active_pane: ActivePane::Input, //Panneau actif
            should_quit: false, //Quitter ?
            screen: ScreenMode::Login, //Panneau par default a l'ouverture du programme
            login_input: String::new(), //Connexion
            scroll_offset: 0, //Position du scroll par defaut
            channels_area: Rect::default(), //Zone du panneau de gauche les salons
            input_area: Rect::default(), //Zone du panneau de droite du chat
            users: Vec::new(),
        }
    }

    //Activer - Desactiver un salon
    pub fn active_channel_name(&self) -> &str{
        &self.channels[self.current_channel]
    }

    //Utilisateur en ligne ou non
    pub fn set_user_online(&mut self, username: &str, online: bool){
        let trimmed = username.trim();
        //Si le nom est vide
        if trimmed.is_empty() || trimmed == "Système" {
            return;
        }
        //Check le status => online
        if let Some((_, is_on_line)) = self.users.iter_mut().find(|(u, _)| u == trimmed){
            *is_on_line = online
        }else{
            self.users.push((trimmed.to_string(), online));
        }
    }

    //Valider et ajouter un message (Salon + auteur + contenu du message)
    pub fn push_message(&mut self, channel: &str, author: String, content: String){
        self.messages
            .entry(channel.to_string())
            .or_default()
            .push(ChatMessage{author, content});
    }

    pub fn on_network_message(&mut self, channel: String,author: String, content: String){
        if author == "Système"{
            if let Some(pseudo) = content.strip_suffix(" s'est déconnecté"){
                self.set_user_online(pseudo, false)
            }
        }else{
            self.set_user_online(&author, true)
        }
        //Si chat est actif
        let is_active = self.active_channel_name() == channel;
        //Ajoute un message => ce truc n'est pas encore lu
        self.push_message(&channel, author, content);
        //Check le scroll des salons
        if is_active{
            self.scroll_offset = 0; //Met en bas du scroll
        }else{
            // ici tu pourras incrémenter le compteur unread du salon
        }
    }

    //Changement de salon: met a jour l'index et remetle scroll en bas
    pub fn select_channel(&mut self, index: usize){
        //Si l'index est > au nombre de salon
        if index < self.channels.len(){
            self.current_channel = index;
            self.scroll_offset = 0; //Met en bas
        }
    }
}
