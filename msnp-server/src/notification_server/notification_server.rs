use crate::{
    Message,
    models::transient::authenticated_user::AuthenticatedUser,
    notification_server::commands::{
        adc::Adc, adg::Adg, blp::Blp, broadcasted_command::BroadcastedCommand, chg::Chg,
        command::Command, cvr::Cvr, fln::Fln, gcf::Gcf, gtc::Gtc, iln::Iln, nln::Nln, prp::Prp,
        reg::Reg, rem::Rem, rmg::Rmg, sbp::Sbp, sdc::Sdc, syn::Syn, ubx::Ubx, url::Url, usr::Usr,
        uux::Uux, ver::Ver, xfr::Xfr,
    },
    switchboard::session::Session,
};
use argon2::password_hash::rand_core::{OsRng, RngCore};
use core::str;
use diesel::{
    MysqlConnection,
    r2d2::{ConnectionManager, Pool},
};
use std::sync::{Arc, Mutex};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, tcp::WriteHalf},
    sync::broadcast,
};

pub struct NotificationServer {
    pool: Pool<ConnectionManager<MysqlConnection>>,
    pub broadcast_tx: broadcast::Sender<Message>,
    contact_rx: Option<broadcast::Receiver<Message>>,
    pub authenticated_user: Option<AuthenticatedUser>,
    protocol_version: Option<String>,
}

impl NotificationServer {
    pub fn new(
        pool: Pool<ConnectionManager<MysqlConnection>>,
        broadcast_tx: broadcast::Sender<Message>,
    ) -> Self {
        NotificationServer {
            pool,
            broadcast_tx: broadcast_tx.clone(),
            contact_rx: None,
            authenticated_user: None,
            protocol_version: None,
        }
    }

    pub async fn listen(&mut self, socket: &mut TcpStream) -> Result<(), &'static str> {
        let (mut rd, mut wr) = socket.split();
        let mut buf = vec![0; 1664];

        if self.authenticated_user.is_some() {
            tokio::select! {
                received = rd.read(&mut buf) => {
                    let received = received.unwrap();
                    if received == 0 {
                        return Err("Client disconnected");
                    }

                    let messages = str::from_utf8(&buf[..received]).unwrap().to_string();
                    self.handle_client_commands(&mut wr, messages).await?
                }

                received = self.contact_rx.as_mut().unwrap().recv() => {
                    self.handle_thread_commands(&mut wr, received.unwrap()).await?
                }
            }
        } else {
            tokio::select! {
                received = rd.read(&mut buf) => {
                    let received = received.unwrap();
                    if received == 0 {
                        return Err("Client disconnected");
                    }

                    let messages = str::from_utf8(&buf[..received]).unwrap().to_string();
                    self.handle_client_commands(&mut wr, messages).await?
                }
            }
        }

        Ok(())
    }

    async fn handle_client_commands(
        &mut self,
        wr: &mut WriteHalf<'_>,
        messages: String,
    ) -> Result<(), &'static str> {
        let mut messages: Vec<String> = messages.lines().map(|line| line.to_string()).collect();

        // Hack to concat payloads and payload commands
        for i in 0..messages.len() - 1 {
            let args: Vec<&str> = messages[i].trim().split(' ').collect();

            match args[0] {
                "UUX" => {
                    let length: usize = args[2].parse().unwrap();
                    let length = messages[i].len() + length;

                    messages[i] = messages[i].clone() + "\r\n" + messages[i + 1].as_str();
                    let next = messages[i].split_off(length + "\r\n".len());

                    if next != "" {
                        messages[i + 1] = next;
                    } else {
                        messages.remove(i + 1);
                    }
                }

                _ => (),
            }
        }

        let messages: Vec<String> = messages
            .iter()
            .map(|msg| msg.to_string() + "\r\n")
            .collect();
        for message in messages {
            println!("C: {message}");
            let command: Vec<&str> = message.trim().split(' ').collect();

            if self.protocol_version.is_none() {
                match command[0] {
                    "VER" => {
                        let responses = match Ver.handle(&message) {
                            Ok(r) => r,
                            Err(err) => {
                                wr.write_all(err.as_bytes()).await.unwrap();
                                println!("S: {err}");
                                return Err("Client uses unsupported MSNP version");
                            }
                        };

                        for reply in responses {
                            wr.write_all(reply.as_bytes()).await.unwrap();
                            println!("S: {reply}");

                            let args: Vec<&str> = reply.trim().split(' ').collect();
                            if args[0] == "VER" {
                                self.protocol_version = Some(args[2].to_string());
                            }
                        }
                    }
                    _ => (),
                }
                continue;
            }

            if self.authenticated_user.is_none() {
                match command[0] {
                    "CVR" => {
                        let responses = Cvr.handle(&message).unwrap();
                        for reply in responses {
                            wr.write_all(reply.as_bytes()).await.unwrap();
                            println!("S: {reply}");
                        }
                    }

                    "USR" => {
                        let mut usr = Usr::new(self.pool.clone());
                        let responses = match usr.handle(&message) {
                            Ok(r) => r,
                            Err(err) => {
                                wr.write_all(err.as_bytes()).await.unwrap();
                                println!("S: {err}");
                                return Err("Disconnecting client");
                            }
                        };

                        for reply in responses {
                            wr.write_all(reply.as_bytes()).await.unwrap();
                            println!("S: {reply}");

                            if reply.contains("OK") && !reply.contains("TWN") {
                                self.broadcast_tx.send(Message::AddUser).unwrap();
                                
                                let user_email = usr.get_user_email().unwrap();
                                self.authenticated_user =
                                    Some(AuthenticatedUser::new(user_email.clone()));

                                let thread_message = Message::ToContact {
                                    sender: user_email.clone(),
                                    receiver: user_email.clone(),
                                    message: "OUT OTH\r\n".to_string(),
                                };

                                self.broadcast_tx.send(thread_message).unwrap();

                                let (tx, _) = broadcast::channel::<Message>(16);
                                self.broadcast_tx
                                    .send(Message::Set {
                                        key: user_email,
                                        value: tx.clone(),
                                    })
                                    .unwrap();

                                self.contact_rx = Some(tx.subscribe());
                            }
                        }
                    }
                    _ => println!("Unmatched command before authentication: {}", message),
                }
                continue;
            }

            match command[0] {
                "USR" => {
                    let tr_id = command[1];
                    let err = format!("207 {tr_id}\r\n");

                    wr.write_all(err.as_bytes()).await.unwrap();
                    println!("S: {err}");
                }

                "SYN" => {
                    let mut syn = Syn::new(
                        self.pool.clone(),
                        self.authenticated_user.as_ref().unwrap().clone(),
                    );

                    let responses = syn.handle(&message).unwrap();
                    for reply in responses {
                        wr.write_all(reply.as_bytes()).await.unwrap();
                        println!("S: {reply}");
                    }

                    self.authenticated_user = Some(syn.authenticated_user);
                }

                "GCF" => {
                    let responses = Gcf.handle(&message).unwrap();
                    for reply in responses {
                        wr.write_all(reply.as_bytes()).await.unwrap();
                        println!("S: {reply}");
                    }
                }

                "URL" => {
                    let responses = Url.handle(&message).unwrap();
                    for reply in responses {
                        wr.write_all(reply.as_bytes()).await.unwrap();
                        println!("S: {reply}");
                    }
                }

                "CHG" => {
                    let first_chg = self.authenticated_user.as_ref().unwrap().presence.is_none();
                    match Chg.handle_with_authenticated_user(
                        &message,
                        self.authenticated_user.as_mut().unwrap(),
                    ) {
                        Ok(responses) => {
                            for reply in responses {
                                wr.write_all(reply.as_bytes()).await.unwrap();
                                println!("S: {reply}");
                            }

                            for email in self.authenticated_user.clone().unwrap().contacts.keys() {
                                if let Some(contact) =
                                    self.authenticated_user.clone().unwrap().contacts.get(email)
                                {
                                    if self.authenticated_user.as_ref().unwrap().blp == "BL"
                                        && !contact.in_allow_list
                                    {
                                        continue;
                                    }

                                    if contact.in_block_list {
                                        continue;
                                    }
                                }

                                if command[2] != "HDN" {
                                    let nln_command = Chg::convert(
                                        &self.authenticated_user.as_ref().unwrap(),
                                        &message,
                                    );

                                    let thread_message = Message::ToContact {
                                        sender: self
                                            .authenticated_user
                                            .as_ref()
                                            .unwrap()
                                            .email
                                            .clone(),
                                        receiver: email.clone(),
                                        message: nln_command,
                                    };

                                    self.broadcast_tx.send(thread_message).unwrap();
                                } else {
                                    let fln_command = Fln::convert(
                                        &self.authenticated_user.as_ref().unwrap(),
                                        &"".to_string(),
                                    );

                                    let message = Message::ToContact {
                                        sender: self
                                            .authenticated_user
                                            .as_ref()
                                            .unwrap()
                                            .email
                                            .clone(),
                                        receiver: email.clone(),
                                        message: fln_command,
                                    };

                                    self.broadcast_tx.send(message).unwrap();
                                    continue;
                                }

                                if first_chg {
                                    let thread_message = Message::ToContact {
                                        sender: self
                                            .authenticated_user
                                            .as_ref()
                                            .unwrap()
                                            .email
                                            .clone(),
                                        receiver: email.clone(),
                                        message: message.clone(),
                                    };

                                    self.broadcast_tx.send(thread_message).unwrap();
                                }
                            }
                        }

                        Err(err) => {
                            wr.write_all(err.as_bytes()).await.unwrap();
                            println!("S: {err}");
                            continue;
                        }
                    };
                }

                "UUX" => {
                    let responses = Uux
                        .handle_with_authenticated_user(
                            &message,
                            self.authenticated_user.as_mut().unwrap(),
                        )
                        .unwrap();
                    for reply in responses {
                        wr.write_all(reply.as_bytes()).await.unwrap();
                        println!("S: {reply}");
                    }

                    // Joining command and payload is done beforehand
                    if let Some(personal_message) = message.lines().nth(1) {
                        self.authenticated_user.as_mut().unwrap().personal_message =
                            Some(personal_message.to_string());
                    }

                    for email in self.authenticated_user.clone().unwrap().contacts.keys() {
                        if let Some(contact) =
                            self.authenticated_user.clone().unwrap().contacts.get(email)
                        {
                            if self.authenticated_user.as_ref().unwrap().blp == "BL"
                                && !contact.in_allow_list
                            {
                                continue;
                            }

                            if contact.in_block_list {
                                continue;
                            }

                            if let Some(presence) =
                                &self.authenticated_user.as_ref().unwrap().presence
                            {
                                if presence == "HDN" {
                                    continue;
                                }
                            }
                        }

                        let ubx_command =
                            Ubx::convert(self.authenticated_user.as_ref().unwrap(), &message);

                        let thread_message = Message::ToContact {
                            sender: self.authenticated_user.as_ref().unwrap().email.clone(),
                            receiver: email.clone(),
                            message: ubx_command,
                        };

                        self.broadcast_tx.send(thread_message).unwrap();
                    }
                }

                "PRP" => {
                    let mut prp = Prp::new(self.pool.clone());

                    match prp.handle_with_authenticated_user(
                        &message,
                        self.authenticated_user.as_mut().unwrap(),
                    ) {
                        Ok(responses) => {
                            for reply in responses {
                                wr.write_all(reply.as_bytes()).await.unwrap();
                                println!("S: {reply}");
                            }
                        }

                        Err(err) => {
                            wr.write_all(err.as_bytes()).await.unwrap();
                            println!("S: {err}");
                        }
                    };
                }

                "SBP" => {
                    let mut sbp = Sbp::new(self.pool.clone());

                    match sbp.handle_with_authenticated_user(
                        &message,
                        self.authenticated_user.as_mut().unwrap(),
                    ) {
                        Ok(responses) => {
                            for reply in responses {
                                wr.write_all(reply.as_bytes()).await.unwrap();
                                println!("S: {reply}");
                            }
                        }

                        Err(err) => {
                            wr.write_all(err.as_bytes()).await.unwrap();
                            println!("S: {err}");
                        }
                    };
                }

                "SDC" => {
                    let responses = Sdc.handle(&message).unwrap();
                    for reply in responses {
                        wr.write_all(reply.as_bytes()).await.unwrap();
                        println!("S: {reply}");
                    }
                }

                "ADC" => {
                    let mut adc = Adc::new(self.pool.clone());

                    match adc.handle_with_authenticated_user(
                        &message,
                        self.authenticated_user.as_mut().unwrap(),
                    ) {
                        Ok(responses) => {
                            for reply in responses {
                                wr.write_all(reply.as_bytes()).await.unwrap();
                                println!("S: {reply}");

                                let args: Vec<&str> = reply.trim().split(' ').collect();
                                if args[2] == "FL" && args[3].starts_with("N=") {
                                    let contact_email = args[3].replace("N=", "");

                                    let reply = Message::ToContact {
                                        sender: self
                                            .authenticated_user
                                            .as_ref()
                                            .unwrap()
                                            .email
                                            .clone(),
                                        receiver: contact_email,
                                        message: Adc::convert(
                                            self.authenticated_user.as_ref().unwrap(),
                                            &message,
                                        ),
                                    };

                                    self.broadcast_tx.send(reply).unwrap();
                                }

                                if args[2] == "BL" && args[3].starts_with("N=") {
                                    let contact_email = args[3].replace("N=", "");

                                    let fln_command = Fln::convert(
                                        &self.authenticated_user.as_ref().unwrap(),
                                        &message,
                                    );

                                    let message = Message::ToContact {
                                        sender: self
                                            .authenticated_user
                                            .as_ref()
                                            .unwrap()
                                            .email
                                            .clone(),
                                        receiver: contact_email.clone(),
                                        message: fln_command,
                                    };

                                    self.broadcast_tx.send(message).unwrap();
                                }
                            }
                        }

                        Err(err) => {
                            wr.write_all(err.as_bytes()).await.unwrap();
                            println!("S: {err}");
                        }
                    };
                }

                "REM" => {
                    let mut rem = Rem::new(self.pool.clone());

                    match rem.handle_with_authenticated_user(
                        &message,
                        self.authenticated_user.as_mut().unwrap(),
                    ) {
                        Ok(responses) => {
                            for reply in responses {
                                wr.write_all(reply.as_bytes()).await.unwrap();
                                println!("S: {reply}");

                                let args: Vec<&str> = reply.trim().split(' ').collect();
                                if args[2] == "FL" {
                                    let contact_email = rem.get_contact_email(args[3]);

                                    let reply = Message::ToContact {
                                        sender: self
                                            .authenticated_user
                                            .as_ref()
                                            .unwrap()
                                            .email
                                            .clone(),
                                        receiver: contact_email,
                                        message: Rem::convert(
                                            self.authenticated_user.as_ref().unwrap(),
                                            &message,
                                        ),
                                    };

                                    self.broadcast_tx.send(reply).unwrap();
                                }

                                if args[2] == "BL" {
                                    let contact_email = command[3].to_string();

                                    let nln_command = Nln::convert(
                                        &self.authenticated_user.as_ref().unwrap(),
                                        &message,
                                    );

                                    let thread_message = Message::ToContact {
                                        sender: self
                                            .authenticated_user
                                            .as_ref()
                                            .unwrap()
                                            .email
                                            .clone(),
                                        receiver: contact_email,
                                        message: nln_command,
                                    };

                                    self.broadcast_tx.send(thread_message).unwrap();
                                }
                            }
                        }

                        Err(err) => {
                            if err == "Removing from RL" {
                                return Err("Disconnecting client");
                            }

                            wr.write_all(err.as_bytes()).await.unwrap();
                            println!("S: {err}");
                        }
                    };
                }

                "ADG" => {
                    let mut adg = Adg::new(self.pool.clone());

                    match adg.handle_with_authenticated_user(
                        &message,
                        self.authenticated_user.as_mut().unwrap(),
                    ) {
                        Ok(responses) => {
                            for reply in responses {
                                wr.write_all(reply.as_bytes()).await.unwrap();
                                println!("S: {reply}");
                            }
                        }

                        Err(err) => {
                            wr.write_all(err.as_bytes()).await.unwrap();
                            println!("S: {err}");
                        }
                    };
                }

                "RMG" => {
                    let mut rmg = Rmg::new(self.pool.clone());

                    match rmg.handle_with_authenticated_user(
                        &message,
                        self.authenticated_user.as_mut().unwrap(),
                    ) {
                        Ok(responses) => {
                            for reply in responses {
                                wr.write_all(reply.as_bytes()).await.unwrap();
                                println!("S: {reply}");
                            }
                        }

                        Err(err) => {
                            if err == "Removing from RL" {
                                return Err("Disconnecting client");
                            }

                            wr.write_all(err.as_bytes()).await.unwrap();
                            println!("S: {err}");
                        }
                    };
                }

                "REG" => {
                    let mut reg = Reg::new(self.pool.clone());

                    match reg.handle_with_authenticated_user(
                        &message,
                        self.authenticated_user.as_mut().unwrap(),
                    ) {
                        Ok(responses) => {
                            for reply in responses {
                                wr.write_all(reply.as_bytes()).await.unwrap();
                                println!("S: {reply}");
                            }
                        }

                        Err(err) => {
                            wr.write_all(err.as_bytes()).await.unwrap();
                            println!("S: {err}");
                        }
                    };
                }

                "BLP" => {
                    let mut blp = Blp::new(self.pool.clone());
                    let responses = blp
                        .handle_with_authenticated_user(
                            &message,
                            self.authenticated_user.as_mut().unwrap(),
                        )
                        .unwrap();

                    for reply in responses {
                        wr.write_all(reply.as_bytes()).await.unwrap();
                        println!("S: {reply}");
                    }
                }

                "GTC" => {
                    let mut gtc = Gtc::new(self.pool.clone());
                    let responses = gtc
                        .handle_with_authenticated_user(
                            &message,
                            self.authenticated_user.as_mut().unwrap(),
                        )
                        .unwrap();

                    for reply in responses {
                        wr.write_all(reply.as_bytes()).await.unwrap();
                        println!("S: {reply}");
                    }
                }

                "XFR" => {
                    match Xfr.handle_with_authenticated_user(
                        &message,
                        self.authenticated_user.as_mut().unwrap(),
                    ) {
                        Ok(responses) => {
                            for reply in responses {
                                let (tx, _) = broadcast::channel::<Message>(16);
                                let args: Vec<&str> = reply.trim().split(' ').collect();
                                let session_id = format!("{:08}", OsRng.next_u32());
                                let cki_string = args[5].to_string();

                                let session = Session {
                                    session_id,
                                    cki_string: cki_string.clone(),
                                    session_tx: tx,
                                    principals: Arc::new(Mutex::new(Vec::new())),
                                };

                                self.broadcast_tx
                                    .send(Message::SetSession {
                                        key: cki_string,
                                        value: session,
                                    })
                                    .unwrap();

                                wr.write_all(reply.as_bytes()).await.unwrap();
                                println!("S: {reply}");
                            }
                        }

                        Err(error) => {
                            wr.write_all(error.as_bytes()).await.unwrap();
                            println!("S: {error}");
                        }
                    };
                }

                "PNG" => {
                    let reply = "QNG 50\r\n";
                    wr.write_all(reply.as_bytes()).await.unwrap();
                    println!("S: {reply}");
                }

                "OUT" => {
                    return Err("Client disconnected");
                }

                _ => println!("Unmatched command: {}", message),
            };
        }

        Ok(())
    }

    async fn handle_thread_commands(
        &mut self,
        wr: &mut WriteHalf<'_>,
        message: Message,
    ) -> Result<(), &'static str> {
        let Message::ToContact {
            sender,
            receiver: _,
            message,
        } = message
        else {
            return Err("Message type must be ToContact");
        };

        println!("Thread {}: {message}", sender);
        let command: Vec<&str> = message.trim().split(' ').collect();
        match command[0] {
            "ILN" => {
                let presence = command[2];
                let contact = command[3];

                if let Some(contact) = self
                    .authenticated_user
                    .as_mut()
                    .unwrap()
                    .contacts
                    .get_mut(contact)
                {
                    contact.presence = Some(presence.to_string());
                }

                wr.write_all(message.as_bytes()).await.unwrap();
                println!("S: {message}");
            }

            "NLN" => {
                if command.len() < 2 {
                    return Ok(());
                }

                let presence = command[1];
                let contact = command[2];

                if let Some(contact) = self
                    .authenticated_user
                    .as_mut()
                    .unwrap()
                    .contacts
                    .get_mut(contact)
                {
                    contact.presence = Some(presence.to_string());
                }

                wr.write_all(message.as_bytes()).await.unwrap();
                println!("S: {message}");
            }

            "FLN" => {
                let contact = command[1].trim();

                if let Some(contact) = self
                    .authenticated_user
                    .as_mut()
                    .unwrap()
                    .contacts
                    .get_mut(contact)
                {
                    contact.presence = None;
                }

                wr.write_all(message.as_bytes()).await.unwrap();
                println!("S: {message}");
            }

            "UBX" => {
                wr.write_all(message.as_bytes()).await.unwrap();
                println!("S: {message}");
            }

            "CHG" => {
                // A user has logged in
                if let Some(contact) = self
                    .authenticated_user
                    .clone()
                    .unwrap()
                    .contacts
                    .get(&sender)
                {
                    if self.authenticated_user.as_ref().unwrap().blp == "BL"
                        && !contact.in_allow_list
                    {
                        return Ok(());
                    }

                    if contact.in_block_list {
                        return Ok(());
                    }

                    if let Some(presence) = &self.authenticated_user.as_ref().unwrap().presence {
                        if presence == "HDN" {
                            return Ok(());
                        }
                    }
                }

                let iln_command = Iln::convert(self.authenticated_user.as_ref().unwrap(), &message);

                let thread_message = Message::ToContact {
                    sender: self.authenticated_user.as_ref().unwrap().email.clone(),
                    receiver: sender.clone(),
                    message: iln_command,
                };

                self.broadcast_tx.send(thread_message).unwrap();

                let ubx_command = Ubx::convert(self.authenticated_user.as_ref().unwrap(), &message);

                let thread_message = Message::ToContact {
                    sender: self.authenticated_user.as_ref().unwrap().email.clone(),
                    receiver: sender,
                    message: ubx_command,
                };

                self.broadcast_tx.send(thread_message).unwrap();
            }

            "ADC" => {
                if let Some(contact) = self
                    .authenticated_user
                    .clone()
                    .unwrap()
                    .contacts
                    .get(&sender)
                {
                    if self.authenticated_user.as_ref().unwrap().blp == "BL"
                        && !contact.in_allow_list
                    {
                        return Ok(());
                    }

                    if contact.in_block_list {
                        return Ok(());
                    }

                    if let Some(presence) = &self.authenticated_user.as_ref().unwrap().presence {
                        if presence == "HDN" {
                            return Ok(());
                        }
                    }
                }

                let nln_command = Nln::convert(self.authenticated_user.as_ref().unwrap(), &message);

                let thread_message = Message::ToContact {
                    sender: self.authenticated_user.as_ref().unwrap().email.clone(),
                    receiver: sender.clone(),
                    message: nln_command,
                };

                self.broadcast_tx.send(thread_message).unwrap();

                let ubx_command = Ubx::convert(self.authenticated_user.as_ref().unwrap(), &message);

                let thread_message = Message::ToContact {
                    sender: self.authenticated_user.as_ref().unwrap().email.clone(),
                    receiver: sender,
                    message: ubx_command,
                };

                self.broadcast_tx.send(thread_message).unwrap();

                wr.write_all(message.as_bytes()).await.unwrap();
                println!("S: {message}");
            }

            "REM" => {
                wr.write_all(message.as_bytes()).await.unwrap();
                println!("S: {message}");
            }

            "RNG" => {
                if let Some(presence) = &self.authenticated_user.as_ref().unwrap().presence {
                    let thread_message = Message::ToContact {
                        sender: self.authenticated_user.as_ref().unwrap().email.clone(),
                        receiver: self.authenticated_user.as_ref().unwrap().email.clone(),
                        message: presence.clone(),
                    };

                    self.broadcast_tx.send(thread_message).unwrap();

                    if presence != "HDN" {
                        wr.write_all(message.as_bytes()).await.unwrap();
                        println!("S: {message}");
                    }
                } else {
                    let thread_message = Message::ToContact {
                        sender: self.authenticated_user.as_ref().unwrap().email.clone(),
                        receiver: self.authenticated_user.as_ref().unwrap().email.clone(),
                        message: "None".to_string(),
                    };

                    self.broadcast_tx.send(thread_message).unwrap();
                }
            }

            "OUT" => {
                wr.write_all(message.as_bytes()).await.unwrap();
                println!("S: {message}");

                return Err("User logged in in another computer");
            }
            _ => (),
        };

        Ok(())
    }

    pub(crate) async fn send_fln_to_contacts(&mut self) {
        for email in self.authenticated_user.clone().unwrap().contacts.keys() {
            let fln_command =
                Fln::convert(&self.authenticated_user.as_ref().unwrap(), &"".to_string());

            let message = Message::ToContact {
                sender: self.authenticated_user.as_ref().unwrap().email.clone(),
                receiver: email.clone(),
                message: fln_command,
            };

            self.broadcast_tx.send(message).unwrap();
        }
    }
}
