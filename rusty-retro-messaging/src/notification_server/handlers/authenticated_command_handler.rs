use super::traits::command_handler::CommandHandler;
use crate::{
    error_command::ErrorCommand,
    message::Message,
    models::transient::authenticated_user::AuthenticatedUser,
    notification_server::commands::{
        adc::Adc, adg::Adg, blp::Blp, chg::Chg, gcf::Gcf, gtc::Gtc, prp::Prp, reg::Reg, rem::Rem,
        rmg::Rmg, sbp::Sbp, sdc::Sdc, syn::Syn, url::Url, uux::Uux, xfr::Xfr,
    },
};
use diesel::{
    MysqlConnection,
    r2d2::{ConnectionManager, Pool},
};
use tokio::{io::AsyncWriteExt, net::tcp::WriteHalf, sync::broadcast};

pub struct AuthenticatedCommandHandler {
    pool: Pool<ConnectionManager<MysqlConnection>>,
    broadcast_tx: broadcast::Sender<Message>,
    pub authenticated_user: AuthenticatedUser,
    protocol_version: usize,
}

impl AuthenticatedCommandHandler {
    pub fn new(
        pool: Pool<ConnectionManager<MysqlConnection>>,
        broadcast_tx: broadcast::Sender<Message>,
        authenticated_user: AuthenticatedUser,
        protocol_version: usize,
    ) -> Self {
        AuthenticatedCommandHandler {
            pool,
            broadcast_tx,
            authenticated_user,
            protocol_version,
        }
    }
}

impl CommandHandler for AuthenticatedCommandHandler {
    async fn handle_command(
        &mut self,
        sender: String,
        wr: &mut WriteHalf<'_>,
        command: String,
    ) -> Result<(), ErrorCommand> {
        let _ = sender;
        let args: Vec<&str> = command.trim().split(' ').collect();

        match args[0] {
            "USR" => {
                let tr_id = args[1];
                let err = format!("207 {tr_id}\r\n");

                wr.write_all(err.as_bytes())
                    .await
                    .expect("Could not send to client over socket");

                println!("S: {err}");
            }

            "SYN" => {
                let mut syn = Syn::new(self.pool.clone());
                Self::run_authenticated_command(
                    self.protocol_version,
                    &mut self.authenticated_user,
                    wr,
                    &mut syn,
                    &command,
                )
                .await?;
            }

            "GCF" => {
                Self::run_command(self.protocol_version, wr, &mut Gcf, &command).await?;
            }

            "URL" => {
                Self::run_command(self.protocol_version, wr, &mut Url, &command).await?;
            }

            "CHG" => {
                let first_chg = self.authenticated_user.presence.is_none();
                let mut chg = Chg::new(self.broadcast_tx.clone(), first_chg);

                Self::run_authenticated_command(
                    self.protocol_version,
                    &mut self.authenticated_user,
                    wr,
                    &mut chg,
                    &command,
                )
                .await?;
            }

            "UUX" => {
                let mut uux = Uux::new(self.broadcast_tx.clone());

                Self::run_authenticated_command(
                    self.protocol_version,
                    &mut self.authenticated_user,
                    wr,
                    &mut uux,
                    &command,
                )
                .await?;
            }

            "PRP" => {
                let mut prp = Prp::new(self.pool.clone());
                Self::run_authenticated_command(
                    self.protocol_version,
                    &mut self.authenticated_user,
                    wr,
                    &mut prp,
                    &command,
                )
                .await?;
            }

            "SBP" => {
                let mut sbp = Sbp::new(self.pool.clone());
                Self::run_authenticated_command(
                    self.protocol_version,
                    &mut self.authenticated_user,
                    wr,
                    &mut sbp,
                    &command,
                )
                .await?;
            }

            "SDC" => {
                Self::run_command(self.protocol_version, wr, &mut Sdc, &command).await?;
            }

            "ADC" => {
                let mut adc = Adc::new(self.pool.clone(), self.broadcast_tx.clone());
                Self::run_authenticated_command(
                    self.protocol_version,
                    &mut self.authenticated_user,
                    wr,
                    &mut adc,
                    &command,
                )
                .await?;
            }

            "REM" => {
                let mut rem = Rem::new(self.pool.clone(), self.broadcast_tx.clone());
                Self::run_authenticated_command(
                    self.protocol_version,
                    &mut self.authenticated_user,
                    wr,
                    &mut rem,
                    &command,
                )
                .await?;
            }

            "ADG" => {
                let mut adg = Adg::new(self.pool.clone());
                Self::run_authenticated_command(
                    self.protocol_version,
                    &mut self.authenticated_user,
                    wr,
                    &mut adg,
                    &command,
                )
                .await?;
            }

            "RMG" => {
                let mut rmg = Rmg::new(self.pool.clone());
                Self::run_authenticated_command(
                    self.protocol_version,
                    &mut self.authenticated_user,
                    wr,
                    &mut rmg,
                    &command,
                )
                .await?;
            }

            "REG" => {
                let mut reg = Reg::new(self.pool.clone());
                Self::run_authenticated_command(
                    self.protocol_version,
                    &mut self.authenticated_user,
                    wr,
                    &mut reg,
                    &command,
                )
                .await?;
            }

            "BLP" => {
                let mut blp = Blp::new(self.pool.clone());
                Self::run_authenticated_command(
                    self.protocol_version,
                    &mut self.authenticated_user,
                    wr,
                    &mut blp,
                    &command,
                )
                .await?;
            }

            "GTC" => {
                let mut gtc = Gtc::new(self.pool.clone());
                Self::run_authenticated_command(
                    self.protocol_version,
                    &mut self.authenticated_user,
                    wr,
                    &mut gtc,
                    &command,
                )
                .await?;
            }

            "XFR" => {
                let mut xfr = Xfr::new(self.broadcast_tx.clone());
                Self::run_authenticated_command(
                    self.protocol_version,
                    &mut self.authenticated_user,
                    wr,
                    &mut xfr,
                    &command,
                )
                .await?;
            }

            "PNG" => {
                let reply = "QNG 50\r\n";
                wr.write_all(reply.as_bytes())
                    .await
                    .expect("Could not send to client over socket");

                println!("S: {reply}");
            }

            "OUT" => return Err(ErrorCommand::Disconnect("Client disconnected".to_string())),

            _ => println!("Unmatched command: {command}"),
        };

        Ok(())
    }
}
