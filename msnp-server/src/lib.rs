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

    let (tx, mut rx) = broadcast::channel::<Message>(16);

    println!("Listening on port 1863");
    tokio::spawn(http::listen(pool.clone(), tx.clone()));
    println!("Listening for HTTP on port 3000");

    let mut channels: HashMap<String, broadcast::Sender<Message>> = HashMap::new();
    let mut sessions: HashMap<String, Session> = HashMap::new();
    let mut user_count: u32 = 0;

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

                            if connection.authenticated_user.is_some() {
                                connection.broadcast_tx.send(Message::RemoveUser).unwrap();
                            }

                            if error != "User logged in in another computer" {
                                if let Some(ref user) = connection.authenticated_user {
                                    connection.broadcast_tx.send(Message::Remove(user.email.clone())).unwrap();
                                    connection.send_fln_to_contacts().await;
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
                        if tx.send(Message::Value { key: key.clone(), value: contact_tx.cloned() }).is_err() {
                            println!("Error sending tx to {}", key);
                        }
                    }

                    Message::Set { key, value } => {
                        channels.insert(key, value);
                    }

                    Message::Remove(key) => {
                        channels.remove(&key);
                    }

                    Message::ToContact { receiver, sender, message } => {
                        let tx = channels.get(&receiver);
                        if let Some(tx) = tx {
                            if tx.send(Message::ToContact { sender: sender, receiver: receiver.clone(), message: message }).is_err() {
                                println!("Error sending to {}", receiver);
                            }
                        }
                    }

                    Message::GetSession(key) => {
                        let session = sessions.get(&key);
                        if tx.send(Message::Session { key: key.clone(), value: session.cloned() }).is_err() {
                            println!("Error sending session to {}", key);
                        }
                    }

                    Message::SetSession { key, value } => {
                        sessions.insert(key, value);
                    }

                    Message::RemoveSession(key) => {
                        sessions.remove(&key);
                    }

                    Message::GetUsers => {
                        if tx.send(Message::UserCount(user_count)).is_err() {
                            println!("Error sending user count");
                        }
                    }

                    Message::AddUser => {
                        user_count += 1;
                    }

                    Message::RemoveUser => {
                        user_count -= 1;
                    }
                    _ => ()
                };
            }
        }
    }
}
