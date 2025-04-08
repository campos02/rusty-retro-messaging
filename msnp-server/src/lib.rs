use diesel::{
    MysqlConnection,
    r2d2::{ConnectionManager, Pool},
};
use dotenvy::dotenv;
use message::Message;
use notification_server::notification_server::NotificationServer;
use std::{collections::HashMap, env};
use switchboard::{session::Session, switchboard::Switchboard};
use tokio::{net::TcpListener, sync::broadcast};

mod http;
mod message;
pub mod models;
mod notification_server;
pub mod schema;
mod switchboard;

/// Starts the MSNP and HTTP servers
pub async fn listen() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let switchboard_port = env::var("SWITCHBOARD_PORT").expect("SWITCHBOARD_PORT not set");

    let manager = ConnectionManager::<MysqlConnection>::new(database_url);
    let pool = Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool");

    let notification_server_listener = TcpListener::bind("0.0.0.0:1863").await.unwrap();
    let switchboard_listener = TcpListener::bind(format!("0.0.0.0:{switchboard_port}"))
        .await
        .unwrap();

    println!("Listening on port 1863");
    tokio::spawn(http::listen(pool.clone()));
    println!("Listening for HTTP on port 3000");

    let (tx, mut rx) = broadcast::channel::<Message>(16);
    let mut channels: HashMap<String, broadcast::Sender<Message>> = HashMap::new();
    let mut sessions: HashMap<String, Session> = HashMap::new();

    loop {
        tokio::select! {
            client = notification_server_listener.accept() => {
                let (mut socket, _) = client.unwrap();
                let pool = pool.clone();
                let tx = tx.clone();

                tokio::spawn(async move {
                    let mut connection = NotificationServer::new(pool, tx);
                    loop {
                        if let Err(error) = connection.listen(&mut socket).await {
                            eprintln!("{error}");

                            if error != "User logged in in another computer" {
                                if let Some(ref user) = connection.authenticated_user {
                                    connection.broadcast_tx.send(Message::Remove(user.email.clone())).unwrap();
                                    connection.send_disconnecting_fln_to_contacts().await;
                                }
                            }

                            break;
                        }
                    }
                });
            }

            client = switchboard_listener.accept() => {
                let (mut socket, _) = client.unwrap();
                let pool = pool.clone();
                let tx = tx.clone();

                tokio::spawn(async move {
                    let mut connection = Switchboard::new(pool, tx);
                    loop {
                        if let Err(error) = connection.listen(&mut socket).await {
                            eprintln!("{error}");

                            if let Some(ref session) = connection.session {
                                connection.broadcast_tx.send(Message::RemoveSession(session.session_id.clone())).unwrap();
                                connection.send_bye_to_principals(false).await;
                            }
                            break;
                        }
                    }
                });
            }

            message = rx.recv() => {
                let message = message.unwrap();
                match message {
                    Message::Get(key) => {
                        let contact_tx = channels.get(&key);
                        tx.send(Message::Value {
                            key,
                            value: contact_tx.cloned(),
                        })
                        .unwrap();
                    }

                    Message::Set { key, value } => {
                        channels.insert(key, value);
                    }

                    Message::Remove(key) => {
                        channels.remove(&key);
                    }

                    Message::GetSession(key) => {
                        let session = sessions.get(&key);
                        tx.send(Message::Session {
                            key,
                            value: session.cloned(),
                        })
                        .unwrap();
                    }

                    Message::SetSession { key, value } => {
                        sessions.insert(key, value);
                    }

                    Message::RemoveSession(key) => {
                        sessions.remove(&key);
                    }
                    _ => ()
                };
            }
        }
    }
}
