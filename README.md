## Les concepts fondamentaux

Avant d'écrire du code, il faut bien séparer la logique réseau, l'état de l'application et l'affichage terminal :

    - Architecture Client / Serveur (ou **P2P**) : Comme Discord, tu as généralement un serveur central qui gère
     les serveurs/salons, l'authentification/pseudo, et diffuse les messages aux clients connectés 
     (souvent via WebSockets avec tokio-tungstenite ou du **TCP** brut avec tokio::net::TcpStream + sérialisation 
      **JSON** / Bincode).

    - Asynchronisme avec tokio : Ton client doit écouter le réseau, lire les frappes clavier et rafraîchir l'écran
     en même temps sans jamais bloquer l'interface.

    - Boucle d'événements et channels (tokio::sync::mpsc) : Au lieu de partager   des verrous complexes partout, 
      on fait circuler des événements typés (enum AppEvent) — frappe clavier, message reçu du serveur, tick de rendu, 
      redimensionnement du terminal — vers une boucle centrale qui met à jour l'état (AppState).

    - Immediate Mode UI avec Ratatui (crossterm comme backend) : Ratatui ne garde pas d'objets UI persistants : 
      à chaque frame, tu lis ton état (AppState) et tu dessines le layout (panneau serveurs/salons à gauche, historique au centre, liste des membres à droite, barre de saisie en bas).

## Les grandes étapes de construction

Voici une progression logique pour avancer :

    - Socle **TUI** & boucle d'événements : Initialise crossterm + Ratatui (en pensant à restaurer le terminal 
     proprement sur Drop ou en cas de panique), découpe l'écran avec Layout::horizontal / Layout::vertical, et gère la saisie texte + navigation au clavier.

    - Protocole de messages (serde) : Définis un enum partagé entre client et serveur 
     (ex. ClientMessage::SendMessage { channel, content } et ServerMessage::NewMessage { author, channel, content, timestamp }), sérialisé en **JSON** ou Bincode.

    - Serveur minimal multi-clients : Monte un serveur tokio qui accepte les connexions, 
     maintient la liste des clients connectés (par exemple avec tokio::sync::broadcast pour diffuser les messages d'un salon) et gère les déconnexions propres.

    - Connexion client ↔ serveur : Connecte le client **TUI** au réseau via une tâche tokio::spawn. 
     Quand un message arrive du réseau, pousse un AppEvent::Server(msg) dans le channel principal pour déclencher un redessin.

    - Fonctionnalités *style Discord* : Ajoute progressivement les serveurs/catégories/salons textuels, l'historique scrollable, 
     l'indicateur d'utilisateur en ligne, les mentions (@pseudo) ou le changement de focus clavier (Tab entre la zone de saisie, les salons et les messages).

## Pièges classiques à anticiper

    - Terminal cassé après un crash : Si ton programme panique alors que le raw mode ou l'alternate screen est actif, 
     ton shell peut rester mal configuré. Utilise un garde **RAII** / Drop et un hook de panic pour toujours exécuter le nettoyage crossterm.

    - Défilement et retour à la ligne : Dans un salon de chat, les messages ont des longueurs variables et peuvent wrap. 
     Prévois un calcul d'offset de scroll propre plutôt qu'un simple index brut.

    - Sécurité minimale : Même pour un prototype local, valide la taille des messages, 
     gère les pseudos vides/doublons et évite de faire confiance aux données brutes envoyées par le client.

## Structure des fichiers 
    chat-tui/
    ├── Cargo.toml
    └── src/
        ├── main.rs     # Point d'entrée, initialisation terminal & boucle principale
        ├── event.rs    # Énumération AppEvent et tâches d'écoute (clavier + réseau)
        ├── state.rs    # État du chat (salons, messages, saisie, focus)
        ├── ui.rs       # Rendu Ratatui (3 zones : salons, messages, saisie)
        └── net.rs      # Envoi / réception réseau (TCP / WebSockets + serde)