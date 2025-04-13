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

    let notification_server_listener = TcpListener::bind("0.0.0.0:1863")
        .await
        .expect("Could not bind Notification Server");

    let switchboard_listener = TcpListener::bind(format!("0.0.0.0:{switchboard_port}"))
        .await
        .expect("Could not bind Switchboard");

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
                let (mut socket, _) = match client {
                    Ok(c) => c,
                    Err(error) => {
                        eprintln!("Could not get socket from accepted Notification Server connection: {error}");
                        continue;
                    }
                };

                let pool = pool.clone();
                let tx = tx.clone();

                tokio::spawn(async move {
                    let mut connection = NotificationServer::new(pool, tx);
                    loop {
                        if let Err(error) = connection.listen(&mut socket).await {
                            eprintln!("{error}");

                            if connection.authenticated_user.is_some() {
                                if let Err(error) = connection.broadcast_tx.send(Message::RemoveUser) {
                                    eprintln!("Could not remove user from count: {error}");
                                }
                            }

                            if error != "User logged in in another computer" {
                                if let Some(ref user) = connection.authenticated_user {
                                    if let Err(error) = connection.broadcast_tx.send(Message::RemoveTx(user.email.clone())) {
                                        eprintln!("Could not remove user tx: {error}");
                                    }
                                    connection.send_fln_to_contacts().await;
                                }
                            }
                            break;
                        }
                    }
                });
            }

            client = switchboard_listener.accept() => {
                let (mut socket, _) = match client {
                    Ok(c) => c,
                    Err(error) => {
                        eprintln!("Could not get socket from accepted Switchboard connection: {error}");
                        continue;
                    }
                };

                let pool = pool.clone();
                let tx = tx.clone();

                tokio::spawn(async move {
                    let mut connection = Switchboard::new(pool, tx);
                    loop {
                        if let Err(error) = connection.listen(&mut socket).await {
                            eprintln!("{error}");

                            if let Some(ref session) = connection.session {
                                if let Err(error) = connection.broadcast_tx.send(Message::RemoveSession(session.session_id.clone())) {
                                    eprintln!("Could not remove session: {error}");
                                }
                                connection.send_bye_to_principals(false).await;
                            }
                            break;
                        }
                    }
                });
            }

            message = rx.recv() => {
                let message = match message {
                    Ok(m) => m,
                    Err(error) => {
                        eprintln!("Could not receive message from thread: {error}");
                        continue;
                    }
                };

                match message {
                    Message::GetTx(key) => {
                        let contact_tx = channels.get(&key);
                        if let Err(error) = tx.send(Message::Tx { key: key.clone(), value: contact_tx.cloned() }) {
                            eprintln!("Could not send tx to {key}: {error}");
                        }
                    }

                    Message::SetTx { key, value } => {
                        channels.insert(key, value);
                    }

                    Message::RemoveTx(key) => {
                        channels.remove(&key);
                    }

                    Message::ToContact { receiver, sender, message } => {
                        let tx = channels.get(&receiver);
                        if let Some(tx) = tx {
                            if let Err(error) = tx.send(Message::ToContact { sender: sender, receiver: receiver.clone(), message: message }) {
                                eprintln!("Could not send message to {receiver}: {error}");
                            }
                        }
                    }

                    Message::GetSession(key) => {
                        let session = sessions.get(&key);
                        if let Err(error) = tx.send(Message::Session { key: key.clone(), value: session.cloned() }) {
                            eprintln!("Could not send session to {key}: {error}");
                        }
                    }

                    Message::SetSession { key, value } => {
                        sessions.insert(key, value);
                    }

                    Message::RemoveSession(key) => {
                        sessions.remove(&key);
                    }

                    Message::GetUsers => {
                        if let Err(error) = tx.send(Message::UserCount(user_count)) {
                            eprintln!("Could not send user count: {error}");
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
