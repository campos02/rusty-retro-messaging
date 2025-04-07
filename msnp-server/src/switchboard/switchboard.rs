use crate::{
    Message,
    models::transient::{authenticated_user::AuthenticatedUser, principal::Principal},
    switchboard::{
        commands::{command::Command, joi::Joi, rng::Rng, usr::Usr},
        session::Session,
    },
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use core::str;
use diesel::{
    MysqlConnection,
    r2d2::{ConnectionManager, Pool},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, tcp::WriteHalf},
    sync::broadcast,
};

use super::commands::bye::Bye;

pub struct Switchboard {
    pool: Pool<ConnectionManager<MysqlConnection>>,
    pub broadcast_tx: broadcast::Sender<Message>,
    broadcast_rx: broadcast::Receiver<Message>,
    pub session: Option<Session>,
    session_rx: Option<broadcast::Receiver<Message>>,
    authenticated_user: Option<AuthenticatedUser>,
}

impl Switchboard {
    pub fn new(
        pool: Pool<ConnectionManager<MysqlConnection>>,
        broadcast_tx: broadcast::Sender<Message>,
    ) -> Self {
        Switchboard {
            pool,
            broadcast_tx: broadcast_tx.clone(),
            broadcast_rx: broadcast_tx.subscribe(),
            session: None,
            session_rx: None,
            authenticated_user: None,
        }
    }

    pub async fn listen(&mut self, socket: &mut TcpStream) -> Result<(), &'static str> {
        let (mut rd, mut wr) = socket.split();
        let mut buf = vec![0; 1664];

        if self.session.is_some() {
            let session_rx = self.session_rx.as_mut().unwrap();

            tokio::select! {
                received = rd.read(&mut buf) => {
                    let received = received.unwrap();
                    if received == 0 {
                        return Err("Client disconnected");
                    }

                    let buf = &buf[..received];
                    self.handle_client_commands(socket, buf.to_vec()).await?
                }

                received = session_rx.recv() => {
                    self.handle_session_message(&mut wr, received.unwrap()).await?
                }
            }
        } else {
            tokio::select! {
                received = rd.read(&mut buf) => {
                    let received = received.unwrap();
                    if received == 0 {
                        return Err("Client disconnected");
                    }

                    let buf = &buf[..received];
                    self.handle_client_commands(socket, buf.to_vec()).await?
                }
            }
        }

        Ok(())
    }

    async fn handle_client_commands(
        &mut self,
        socket: &mut TcpStream,
        mut messages_bytes: Vec<u8>,
    ) -> Result<(), &'static str> {
        let (mut rd, mut wr) = socket.split();
        let mut base64_messages: Vec<String> = Vec::new();

        loop {
            let messages_string = unsafe { str::from_utf8_unchecked(&messages_bytes) };
            let messages: Vec<String> = messages_string
                .lines()
                .map(|line| line.to_string() + "\r\n")
                .collect();

            if messages.len() == 0 {
                break;
            }

            let args: Vec<&str> = messages[0].trim().split(' ').collect();
            match args[0] {
                "MSG" => {
                    let length: usize = args[3].parse().unwrap();
                    let length: usize = messages[0].len() + length;

                    if length > messages_bytes.len() {
                        println!("Fetching more message data...");
                        let mut buf = vec![0; 1664];
                        let received = rd.read(&mut buf).await.unwrap();
                        let mut buf = buf[..received].to_vec();

                        messages_bytes.append(&mut buf);
                        continue;
                    }

                    let new_bytes = messages_bytes[..length].to_vec();
                    messages_bytes = messages_bytes[length..].to_vec();

                    let base64_message = URL_SAFE.encode(&new_bytes);
                    base64_messages.push(base64_message);
                }

                _ => {
                    let new_bytes = messages_bytes[..messages[0].len()].to_vec();
                    messages_bytes = messages_bytes[messages[0].len()..].to_vec();

                    let base64_message = URL_SAFE.encode(&new_bytes);
                    base64_messages.push(base64_message);
                }
            }
        }

        for base64_message in base64_messages {
            let bytes = URL_SAFE.decode(base64_message.clone()).unwrap();
            let message = unsafe { str::from_utf8_unchecked(&bytes) };
            let message = message.lines().next().unwrap().to_string() + "\r\n";
            let command: Vec<&str> = message.trim().split(' ').collect();
            println!("C: {}", message);

            if self.session.is_none() {
                match command[0] {
                    "USR" => {
                        let tr_id = command[1];
                        let user_email = command[2];
                        let cki_string = command[3];

                        self.broadcast_tx
                            .send(Message::GetSession(cki_string.to_string()))
                            .unwrap();

                        while let Ok(message) = self.broadcast_rx.recv().await {
                            if let Message::Session { key, value } = message {
                                if key == cki_string {
                                    self.session = value;
                                    break;
                                }
                            }
                        }

                        if self.session.is_none() {
                            let reply = format!("911 {tr_id}\r\n");
                            wr.write_all(reply.as_bytes()).await.unwrap();
                            println!("S: {reply}");

                            return Err("Session not found");
                        }

                        self.session_rx =
                            Some(self.session.as_ref().unwrap().session_tx.subscribe());
                        self.authenticated_user =
                            Some(AuthenticatedUser::new(user_email.to_string()));

                        let reply = Usr.generate(
                            self.pool.clone(),
                            self.authenticated_user.as_mut().unwrap(),
                            tr_id,
                        );

                        {
                            let mut principals =
                                self.session.as_ref().unwrap().principals.lock().unwrap();

                            principals.push(Principal {
                                email: user_email.to_string(),
                                display_name: self
                                    .authenticated_user
                                    .as_ref()
                                    .unwrap()
                                    .display_name
                                    .clone(),
                            });
                        }

                        wr.write_all(reply.as_bytes()).await.unwrap();
                        println!("S: {reply}");
                    }

                    "ANS" => {
                        let tr_id = command[1];
                        let user_email = command[2];
                        let cki_string = command[3];

                        self.broadcast_tx
                            .send(Message::GetSession(cki_string.to_string()))
                            .unwrap();

                        while let Ok(message) = self.broadcast_rx.recv().await {
                            if let Message::Session { key, value } = message {
                                if key == cki_string {
                                    self.session = value;
                                    break;
                                }
                            }
                        }

                        if self.session.is_none() {
                            let reply = format!("911 {tr_id}\r\n");
                            wr.write_all(reply.as_bytes()).await.unwrap();
                            println!("S: {reply}");

                            return Err("Session not found");
                        }

                        self.session_rx =
                            Some(self.session.as_ref().unwrap().session_tx.subscribe());
                        self.authenticated_user =
                            Some(AuthenticatedUser::new(user_email.to_string()));

                        let joi = Joi.generate(
                            self.pool.clone(),
                            self.authenticated_user.as_mut().unwrap(),
                            tr_id,
                        );

                        let mut iro_replies: Vec<String> = Vec::new();

                        {
                            let mut principals =
                                self.session.as_ref().unwrap().principals.lock().unwrap();

                            let count = principals.len();
                            let mut index = 1;
                            for principal in principals.to_vec() {
                                let email = principal.email;
                                let display_name = principal.display_name;

                                iro_replies.push(format!(
                                    "IRO {tr_id} {index} {count} {email} {display_name}\r\n"
                                ));
                                index += 1;
                            }

                            principals.push(Principal {
                                email: user_email.to_string(),
                                display_name: self
                                    .authenticated_user
                                    .as_ref()
                                    .unwrap()
                                    .display_name
                                    .clone(),
                            });
                        }

                        let message = Message::ToPrincipals {
                            sender: user_email.to_string(),
                            message: URL_SAFE.encode(joi.as_bytes()),
                            disconnecting: false,
                        };

                        self.send_to_session(message).await;

                        for reply in iro_replies {
                            wr.write_all(reply.as_bytes()).await.unwrap();
                            println!("S: {reply}");
                        }

                        let reply = format!("ANS {tr_id} OK\r\n");
                        wr.write_all(reply.as_bytes()).await.unwrap();
                        println!("S: {reply}");
                    }
                    _ => println!("Unmatched command before authentication: {}", message),
                }
                continue;
            }

            match command[0] {
                "USR" => {
                    let tr_id = command[1];
                    let err = format!("911 {tr_id}\r\n");

                    println!("S: {err}");
                    wr.write_all(err.as_bytes()).await.unwrap();
                }

                "ANS" => {
                    let tr_id = command[1];
                    let err = format!("911 {tr_id}\r\n");

                    println!("S: {err}");
                    wr.write_all(err.as_bytes()).await.unwrap();
                }

                "CAL" => {
                    let tr_id = command[1];
                    let email = command[2];
                    let session_id = self.session.as_ref().unwrap().session_id.clone();

                    let mut rng = Rng {
                        session_id: session_id.clone(),
                        cki_string: self.session.as_ref().unwrap().cki_string.clone(),
                    };

                    let rng = rng.generate(
                        self.pool.clone(),
                        self.authenticated_user.as_mut().unwrap(),
                        tr_id,
                    );

                    let message = Message::ToContact {
                        sender: email.to_string(),
                        message: rng,
                        disconnecting: false,
                    };

                    if let Err(err) = self.invite_to_session(&email.to_string(), message).await {
                        let reply = format!("{err} {tr_id}\r\n");
                        wr.write_all(reply.as_bytes()).await.unwrap();
                        println!("S: {reply}");
                    } else {
                        let reply = format!("CAL {tr_id} RINGING {session_id}\r\n");
                        wr.write_all(reply.as_bytes()).await.unwrap();
                        println!("S: {reply}");
                    }
                }

                "MSG" => {
                    let email = self.authenticated_user.as_ref().unwrap().email.clone();
                    let display_name = &self.authenticated_user.as_ref().unwrap().display_name;
                    let length = command[3];

                    let async_msg = format!("MSG {email} {display_name} {length}\r\n")
                        .as_bytes()
                        .to_vec();

                    let mut bytes = URL_SAFE.decode(base64_message).unwrap();
                    bytes.splice(..message.as_bytes().len(), async_msg);
                    let base64_message = URL_SAFE.encode(bytes);

                    let message = Message::ToPrincipals {
                        sender: email,
                        message: base64_message,
                        disconnecting: false,
                    };

                    self.send_to_session(message).await;

                    if command[2] == "A" || command[2] == "D" {
                        let tr_id = command[1];
                        let reply = format!("ACK {tr_id}\r\n");
                        wr.write_all(reply.as_bytes()).await.unwrap();
                        println!("S: {reply}");
                    }
                }

                "OUT" => {
                    return Err("Client disconnected");
                }

                _ => println!("Unmatched command: {}", message),
            };
        }

        Ok(())
    }

    async fn handle_session_message(
        &mut self,
        wr: &mut WriteHalf<'_>,
        message: Message,
    ) -> Result<(), &'static str> {
        let Message::ToPrincipals {
            sender,
            message,
            disconnecting: _,
        } = message
        else {
            return Err("Message type must be ToPrincipals");
        };

        let message = URL_SAFE.decode(message).unwrap();
        let messages_string = unsafe { str::from_utf8_unchecked(&message) };
        let command = messages_string.lines().next().unwrap().to_string() + "\r\n";

        let args: Vec<&str> = command.trim().split(' ').collect();

        let principal = args[1];
        if principal == self.authenticated_user.as_ref().unwrap().email {
            return Ok(());
        }

        println!("Thread {}: {command}", sender);

        match args[0] {
            "MSG" => {
                wr.write_all(&message).await.unwrap();
                println!("S: {command}");
            }

            "JOI" => {
                wr.write_all(&message).await.unwrap();
                println!("S: {command}");
            }

            "BYE" => {
                wr.write_all(&message).await.unwrap();
                println!("S: {command}");
            }
            _ => (),
        };

        Ok(())
    }

    async fn send_to_session(&mut self, message: Message) {
        if let Some(ref session) = self.session {
            session.session_tx.send(message).unwrap();
        }
    }

    async fn invite_to_session(&mut self, email: &String, message: Message) -> Result<(), &str> {
        {
            let principals = self.session.as_ref().unwrap().principals.lock().unwrap();
            let user_index = principals
                .iter()
                .position(|principal| principal.email == email.clone());

            if user_index.is_some() {
                return Err("215");
            }
        }

        self.broadcast_tx.send(Message::Get(email.clone())).unwrap();

        let mut contact_tx: Option<broadcast::Sender<Message>> = None;
        while let Ok(message) = self.broadcast_rx.recv().await {
            if let Message::Value { key, value } = message {
                if key == *email {
                    contact_tx = value;
                    break;
                }
            }
        }

        if contact_tx.is_none() {
            return Err("217");
        }

        let mut contact_rx = contact_tx.as_ref().unwrap().subscribe();
        contact_tx.unwrap().send(message).unwrap();

        while let Ok(message) = contact_rx.recv().await {
            if let Message::ToContact {
                sender,
                message,
                disconnecting: _,
            } = message
            {
                if sender == *email {
                    if message == "HDN" || message == "None" {
                        return Err("217");
                    }
                    break;
                }
            }
        }

        Ok(())
    }

    pub(crate) async fn send_bye_to_principals(&mut self, idling: bool) {
        let user_email = self.authenticated_user.as_ref().unwrap().email.clone();

        {
            let mut principals = self.session.as_ref().unwrap().principals.lock().unwrap();
            let user_index = principals
                .iter()
                .position(|principal| principal.email == user_email)
                .unwrap();
            principals.swap_remove(user_index);
        }

        let mut bye_command = Bye.generate(
            self.pool.clone(),
            self.authenticated_user.as_mut().unwrap(),
            "",
        );

        if idling {
            bye_command = bye_command.replace("\r\n", " 1\r\n");
        }

        let message = Message::ToPrincipals {
            sender: user_email,
            message: URL_SAFE.encode(bye_command.as_bytes()),
            disconnecting: true,
        };

        self.send_to_session(message).await;
    }
}
