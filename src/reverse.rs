use tokio::net::{TcpListener, TcpStream};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::spawn; // Correction pour l'importation de spawn
use std::sync::Arc;
use std::error::Error;
use rand::prelude::*; //Importation correcte pour utiliser SliceRandom et thread_rng


pub async fn start_reverse_proxy() -> io::Result<()> {
    let servers = vec!["127.0.0.1:8081","127.0.0.1:8082"];

    let servers = Arc::new(servers);
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("ecoute socket port 8080");
    
    loop{
        let (socket, _) = listener.accept().await?;
        let servers = Arc::clone(&servers);

        spawn (async move {
            if let Err(e) = handle_connection(socket, servers).await{
                println!("Erreur  dans la gestion de connexion: {}", e)
            }
        });
    }
}

async fn handle_connection (mut socket: TcpStream, servers : Arc<Vec<&str>>) -> Result<(), Box<dyn std::error::Error>> { // permet d'avoir des vecteur de serveur en clonant les references de ces derniers, ce ne sont pas de copie de reel donnee
    
    let mut rng = rand::thread_rng();
    let server_choice = servers.as_ref().choose(&mut rng).ok_or("Auccun server disponible")?; // selectionne aleatoirement un serveur et je dois d'abord déréférencer l'Arc pour obtenir une référence à la Vec qu'il contient.
        match TcpStream::connect(server_choice).await { // etablir une connection avec le serveur en amont selectionne
            Ok(mut upstream_socket) =>{
                let mut buffer = [0u8; 1024];
                let bytes_read = socket.read(&mut buffer).await?;
                upstream_socket.write_all(&buffer[..bytes_read]).await?;

                let mut response = Vec::new(); // lire la reponse du server en amont
                upstream_socket.read_to_end(&mut response).await?; 
                socket.write_all(&response).await?;
            },
            Err(_) => {
                // Si la connexion échoue, renvoyez une réponse 502 Bad Gateway au client
                let response = "HTTP/1.1 502 Bad Gateway\r\n\r\n";
                socket.write_all(response.as_bytes()).await?;
            }
    }
    Ok(())
}