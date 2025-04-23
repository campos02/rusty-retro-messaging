use crate::{
    Message,
    models::transient::{authenticated_user::AuthenticatedUser, principal::Principal},
    switchboard::{
        commands::{bye::Bye, joi::Joi, rng::Rng, traits::command::Command, usr::Usr},
        session::Session,
    },
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use core::str;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, tcp::WriteHalf},
    sync::broadcast::{self, error::RecvError},
};

enum InvitationError {
    PrincipalUserNotFound,
    PrincipalOffline,
}

pub struct Switchboard {
    pub broadcast_tx: broadcast::Sender<Message>,
    pub session: Option<Session>,
    session_rx: Option<broadcast::Receiver<Message>>,
    authenticated_user: Option<AuthenticatedUser>,
    protocol_version: Option<usize>,
}

impl Switchboard {
    pub fn new(broadcast_tx: broadcast::Sender<Message>) -> Self {
        Switchboard {
            broadcast_tx: broadcast_tx.clone(),
            session: None,
            session_rx: None,
            authenticated_user: None,
            protocol_version: None,
        }
    }

    pub async fn listen(&mut self, socket: &mut TcpStream) -> Result<(), &'static str> {
        let (mut rd, mut wr) = socket.split();
        let mut buf = vec![0; 1664];

        if self.session.is_some() {
            let session_rx = self
                .session_rx
                .as_mut()
                .expect("Could not get session receiver");

            tokio::select! {
                received = rd.read(&mut buf) => {
                    let Ok(received) = received else {
                        return Err("Could not read from client");
                    };

                    if received == 0 {
                        return Err("Client disconnected");
                    }

                    let buf = &buf[..received];
                    self.handle_client_commands(socket, buf.to_vec()).await?
                }

                received = session_rx.recv() => {
                    self.handle_session_message(&mut wr, received.expect("Could not receive from threads")).await?
                }
            }
        } else {
            tokio::select! {
                received = rd.read(&mut buf) => {
                    let Ok(received) = received else {
                        return Err("Could not read from client");
                    };

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
                    let Ok(length) = args[3].parse::<usize>() else {
                        let tr_id = args[1];
                        let reply = format!("282 {tr_id}\r\n");

                        wr.write_all(reply.as_bytes())
                            .await
                            .expect("Could not send to client over socket");

                        println!("S: {reply}");
                        return Ok(());
                    };

                    let length = messages[0].len() + length;

                    if length > messages_bytes.len() {
                        println!("Fetching more message data...");
                        let mut buf = vec![0; 1664];
                        let Ok(received) = rd.read(&mut buf).await else {
                            return Err("Could not read from client");
                        };

                        if received == 0 {
                            return Err("Client disconnected");
                        }

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
            let bytes = URL_SAFE
                .decode(base64_message.clone())
                .expect("Could not decode client message converted to base64");

            let message = unsafe { str::from_utf8_unchecked(&bytes) };
            let message = message
                .lines()
                .next()
                .expect("Could not get command from client message")
                .to_string()
                + "\r\n";

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
                            .expect("Could not send to broadcast");

                        {
                            let mut broadcast_rx = self.broadcast_tx.subscribe();
                            loop {
                                let message = match broadcast_rx.recv().await {
                                    Ok(msg) => msg,
                                    Err(err) => {
                                        if let RecvError::Lagged(_) = err {
                                            continue;
                                        } else {
                                            return Err("Could not receive from broadcast");
                                        }
                                    }
                                };

                                if let Message::Session { key, value } = message {
                                    if key == cki_string {
                                        self.session = value;

                                        if !broadcast_rx.is_empty() {
                                            continue;
                                        }
                                        break;
                                    }
                                }
                            }
                        }

                        if self.session.is_none() {
                            let reply = format!("911 {tr_id}\r\n");
                            wr.write_all(reply.as_bytes())
                                .await
                                .expect("Could not send to client over socket");

                            println!("S: {reply}");
                            return Err("Session not found");
                        }

                        self.session_rx = Some(
                            self.session
                                .as_ref()
                                .expect("Could not get session")
                                .session_tx
                                .subscribe(),
                        );

                        let message = Message::ToContact {
                            sender: user_email.to_string(),
                            receiver: user_email.to_string(),
                            message: "GetUserDetails".to_string(),
                        };

                        self.broadcast_tx
                            .send(message)
                            .expect("Could not send to broadcast");

                        {
                            let mut broadcast_rx = self.broadcast_tx.subscribe();
                            loop {
                                let message = match broadcast_rx.recv().await {
                                    Ok(msg) => msg,
                                    Err(err) => {
                                        if let RecvError::Lagged(_) = err {
                                            continue;
                                        } else {
                                            return Err("Could not receive from broadcast");
                                        }
                                    }
                                };

                                if let Message::UserDetails {
                                    sender,
                                    receiver: _,
                                    authenticated_user,
                                    protocol_version,
                                } = message
                                {
                                    if sender == user_email {
                                        self.authenticated_user = authenticated_user;
                                        self.protocol_version = protocol_version;

                                        if !broadcast_rx.is_empty() {
                                            continue;
                                        }
                                        break;
                                    }
                                }
                            }
                        }

                        let reply = Usr.generate(
                            self.protocol_version
                                .expect("Could not get protocol version"),
                            self.authenticated_user
                                .as_mut()
                                .expect("Could not get authenticated user"),
                            tr_id,
                        );

                        {
                            let mut principals = self
                                .session
                                .as_ref()
                                .expect("Could not get session")
                                .principals
                                .lock()
                                .expect("Could not get principals, mutex poisoned");

                            principals.push(Principal {
                                email: user_email.to_string(),
                                display_name: self
                                    .authenticated_user
                                    .as_ref()
                                    .expect("Could not get authenticated user")
                                    .display_name
                                    .clone(),
                                client_id: self
                                    .authenticated_user
                                    .as_ref()
                                    .expect("Could not get authenticated user")
                                    .client_id
                                    .clone(),
                            });
                        }

                        wr.write_all(reply.as_bytes())
                            .await
                            .expect("Could not send to client over socket");

                        println!("S: {reply}");
                    }

                    "ANS" => {
                        let tr_id = command[1];
                        let user_email = command[2];
                        let cki_string = command[3];

                        self.broadcast_tx
                            .send(Message::GetSession(cki_string.to_string()))
                            .expect("Could not send to broadcast");

                        {
                            let mut broadcast_rx = self.broadcast_tx.subscribe();
                            loop {
                                let message = match broadcast_rx.recv().await {
                                    Ok(msg) => msg,
                                    Err(err) => {
                                        if let RecvError::Lagged(_) = err {
                                            continue;
                                        } else {
                                            return Err("Could not receive from broadcast");
                                        }
                                    }
                                };

                                if let Message::Session { key, value } = message {
                                    if key == cki_string {
                                        self.session = value;

                                        if !broadcast_rx.is_empty() {
                                            continue;
                                        }
                                        break;
                                    }
                                }
                            }
                        }

                        if self.session.is_none() {
                            let reply = format!("911 {tr_id}\r\n");
                            wr.write_all(reply.as_bytes())
                                .await
                                .expect("Could not send to client over socket");

                            println!("S: {reply}");
                            return Err("Session not found");
                        }

                        self.session_rx = Some(
                            self.session
                                .as_ref()
                                .expect("Could not get session")
                                .session_tx
                                .subscribe(),
                        );

                        let message = Message::ToContact {
                            sender: user_email.to_string(),
                            receiver: user_email.to_string(),
                            message: "GetUserDetails".to_string(),
                        };

                        self.broadcast_tx
                            .send(message)
                            .expect("Could not send to broadcast");

                        {
                            let mut broadcast_rx = self.broadcast_tx.subscribe();
                            loop {
                                let message = match broadcast_rx.recv().await {
                                    Ok(msg) => msg,
                                    Err(err) => {
                                        if let RecvError::Lagged(_) = err {
                                            continue;
                                        } else {
                                            return Err("Could not receive from broadcast");
                                        }
                                    }
                                };

                                if let Message::UserDetails {
                                    sender,
                                    receiver: _,
                                    authenticated_user,
                                    protocol_version,
                                } = message
                                {
                                    if sender == user_email {
                                        self.authenticated_user = authenticated_user;
                                        self.protocol_version = protocol_version;

                                        if !broadcast_rx.is_empty() {
                                            continue;
                                        }
                                        break;
                                    }
                                }
                            }
                        }

                        let joi = Joi.generate(
                            self.protocol_version
                                .expect("Could not get protocol version"),
                            self.authenticated_user
                                .as_mut()
                                .expect("Could not get authenticated user"),
                            tr_id,
                        );

                        let mut iro_replies: Vec<String> = Vec::new();

                        {
                            let mut principals = self
                                .session
                                .as_ref()
                                .expect("Could not get session")
                                .principals
                                .lock()
                                .expect("Could not get principals, mutex poisoned");

                            let count = principals.len();
                            let mut index = 1;
                            for principal in principals.to_vec() {
                                let email = principal.email;
                                let display_name = principal.display_name;

                                let mut iro_reply = format!(
                                    "IRO {tr_id} {index} {count} {email} {display_name}\r\n"
                                );

                                if self
                                    .protocol_version
                                    .expect("Could not get protocol version")
                                    >= 12
                                {
                                    if let Some(client_id) = principal.client_id {
                                        iro_reply = format!(
                                            "IRO {tr_id} {index} {count} {email} {display_name} {client_id}\r\n"
                                        );
                                    }
                                }

                                iro_replies.push(iro_reply);
                                index += 1;
                            }

                            principals.push(Principal {
                                email: user_email.to_string(),
                                display_name: self
                                    .authenticated_user
                                    .as_ref()
                                    .expect("Could not get authenticated user")
                                    .display_name
                                    .clone(),
                                client_id: self
                                    .authenticated_user
                                    .as_ref()
                                    .expect("Could not get authenticated user")
                                    .client_id
                                    .clone(),
                            });
                        }

                        let message = Message::ToPrincipals {
                            sender: user_email.to_string(),
                            message: URL_SAFE.encode(joi.as_bytes()),
                        };

                        self.send_to_session(message).await;

                        for reply in iro_replies {
                            wr.write_all(reply.as_bytes())
                                .await
                                .expect("Could not send to client over socket");

                            println!("S: {reply}");
                        }

                        let reply = format!("ANS {tr_id} OK\r\n");
                        wr.write_all(reply.as_bytes())
                            .await
                            .expect("Could not send to client over socket");

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

                    wr.write_all(err.as_bytes())
                        .await
                        .expect("Could not send to client over socket");

                    println!("S: {err}");
                }

                "ANS" => {
                    let tr_id = command[1];
                    let err = format!("911 {tr_id}\r\n");

                    wr.write_all(err.as_bytes())
                        .await
                        .expect("Could not send to client over socket");

                    println!("S: {err}");
                }

                "CAL" => {
                    let tr_id = command[1];
                    let email = command[2];
                    let session_id = self
                        .session
                        .as_ref()
                        .expect("Could not get session")
                        .session_id
                        .clone();

                    let rng = Rng {
                        session_id: session_id.clone(),
                        cki_string: self
                            .session
                            .as_ref()
                            .expect("Could not get session")
                            .cki_string
                            .clone(),
                    };

                    let rng = rng.generate(
                        self.protocol_version
                            .expect("Could not get protocol version"),
                        self.authenticated_user
                            .as_mut()
                            .expect("Could not get authenticated user"),
                        tr_id,
                    );

                    let message = Message::ToContact {
                        sender: self
                            .authenticated_user
                            .as_ref()
                            .expect("Could not get authenticated user")
                            .email
                            .clone(),
                        receiver: email.to_string(),
                        message: rng,
                    };

                    if let Err(err) = self.invite_to_session(&email.to_string(), message).await {
                        let reply = format!("{err} {tr_id}\r\n");
                        wr.write_all(reply.as_bytes())
                            .await
                            .expect("Could not send to client over socket");

                        println!("S: {reply}");
                    } else {
                        let reply = format!("CAL {tr_id} RINGING {session_id}\r\n");
                        wr.write_all(reply.as_bytes())
                            .await
                            .expect("Could not send to client over socket");

                        println!("S: {reply}");
                    }
                }

                "MSG" => {
                    let email = self
                        .authenticated_user
                        .as_ref()
                        .expect("Could not get authenticated user")
                        .email
                        .clone();

                    let display_name = &self
                        .authenticated_user
                        .as_ref()
                        .expect("Could not get authenticated user")
                        .display_name;

                    let length = command[3];

                    let async_msg = format!("MSG {email} {display_name} {length}\r\n")
                        .as_bytes()
                        .to_vec();

                    let mut bytes = URL_SAFE
                        .decode(base64_message)
                        .expect("Could not decode client message from base64");
                    bytes.splice(..message.as_bytes().len(), async_msg);
                    let base64_message = URL_SAFE.encode(bytes);

                    let message = Message::ToPrincipals {
                        sender: email,
                        message: base64_message,
                    };

                    self.send_to_session(message).await;

                    if command[2] == "A" || command[2] == "D" {
                        let tr_id = command[1];
                        let reply = format!("ACK {tr_id}\r\n");

                        wr.write_all(reply.as_bytes())
                            .await
                            .expect("Could not send to client over socket");

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
        let Message::ToPrincipals { sender, message } = message else {
            return Err("Message type must be ToPrincipals");
        };

        let message = URL_SAFE
            .decode(message)
            .expect("Could not decode base64 from session");
        let messages_string = unsafe { str::from_utf8_unchecked(&message) };
        let command = messages_string
            .lines()
            .next()
            .expect("Could not get command from session message")
            .to_string()
            + "\r\n";

        let args: Vec<&str> = command.trim().split(' ').collect();

        let principal = args[1];
        if principal
            == self
                .authenticated_user
                .as_ref()
                .expect("Could not get authenticated user")
                .email
        {
            return Ok(());
        }

        println!("Thread {}: {command}", sender);

        match args[0] {
            "MSG" => {
                wr.write_all(&message)
                    .await
                    .expect("Could not send to client over socket");

                println!("S: {command}");
            }

            "JOI" => {
                wr.write_all(&message)
                    .await
                    .expect("Could not send to client over socket");

                println!("S: {command}");
            }

            "BYE" => {
                wr.write_all(&message)
                    .await
                    .expect("Could not send to client over socket");

                println!("S: {command}");
            }
            _ => (),
        };

        Ok(())
    }

    async fn send_to_session(&mut self, message: Message) {
        if let Some(ref session) = self.session {
            session
                .session_tx
                .send(message)
                .expect("Could not send to session");
        }
    }

    async fn invite_to_session(&mut self, email: &String, rng: Message) -> Result<(), &str> {
        {
            let principals = self
                .session
                .as_ref()
                .expect("Could not get session")
                .principals
                .lock()
                .expect("Could not get principals, mutex poisoned");

            let user_index = principals
                .iter()
                .position(|principal| principal.email == email.clone());

            if user_index.is_some() {
                return Err("215");
            }
        }

        let message = Message::ToContact {
            sender: self
                .authenticated_user
                .as_ref()
                .expect("Could not get authenticated user")
                .email
                .clone(),
            receiver: email.to_string(),
            message: "GetUserDetails".to_string(),
        };

        self.broadcast_tx
            .send(message)
            .expect("Could not send to broadcast");

        let mut principal_user;

        {
            let mut broadcast_rx = self.broadcast_tx.subscribe();
            loop {
                let message = match broadcast_rx.recv().await {
                    Ok(msg) => msg,
                    Err(err) => {
                        if let RecvError::Lagged(_) = err {
                            continue;
                        } else {
                            return Err("Could not receive from broadcast");
                        }
                    }
                };

                if let Message::UserDetails {
                    sender,
                    receiver: _,
                    authenticated_user,
                    protocol_version: _,
                } = message
                {
                    if sender == *email {
                        principal_user = authenticated_user;

                        if !broadcast_rx.is_empty() {
                            continue;
                        }
                        break;
                    }
                }
            }
        }

        if let Ok(presence) = principal_user
            .ok_or_else(|| InvitationError::PrincipalUserNotFound)
            .and_then(|authenticated_user| {
                authenticated_user
                    .presence
                    .ok_or_else(|| InvitationError::PrincipalOffline)
            })
        {
            if presence == "HDN" {
                return Err("217");
            }
        } else {
            return Err("217");
        }

        if self.broadcast_tx.send(rng).is_err() {
            return Err("217");
        }

        Ok(())
    }

    pub(crate) async fn send_bye_to_principals(&mut self, idling: bool) {
        let user_email = self
            .authenticated_user
            .as_ref()
            .expect("Could not get authenticated user")
            .email
            .clone();
        {
            let mut principals = self
                .session
                .as_ref()
                .expect("Could not get session")
                .principals
                .lock()
                .expect("Could not get principals, mutex poisoned");

            let user_index = principals
                .iter()
                .position(|principal| principal.email == user_email)
                .expect("Could not find user among principals");

            principals.swap_remove(user_index);
        }

        let mut bye_command = Bye.generate(
            self.protocol_version
                .expect("Could not get protocol version"),
            self.authenticated_user
                .as_mut()
                .expect("Could not get authenticated user"),
            "",
        );

        if idling {
            bye_command = bye_command.replace("\r\n", " 1\r\n");
        }

        let message = Message::ToPrincipals {
            sender: user_email,
            message: URL_SAFE.encode(bye_command.as_bytes()),
        };

        self.send_to_session(message).await;
    }
}
