use super::command_handler::CommandHandler;
use super::command_processor::CommandProcessor;
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
use log::{trace, warn};
use tokio::{io::AsyncWriteExt, net::tcp::WriteHalf, sync::broadcast};

pub struct UserCommandHandler {
    pool: Pool<ConnectionManager<MysqlConnection>>,
    broadcast_tx: broadcast::Sender<Message>,
    pub authenticated_user: AuthenticatedUser,
    protocol_version: usize,
}

impl UserCommandHandler {
    pub fn new(
        pool: Pool<ConnectionManager<MysqlConnection>>,
        broadcast_tx: broadcast::Sender<Message>,
        authenticated_user: AuthenticatedUser,
        protocol_version: usize,
    ) -> Self {
        UserCommandHandler {
            pool,
            broadcast_tx,
            authenticated_user,
            protocol_version,
        }
    }
}

impl CommandHandler for UserCommandHandler {
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

                warn!("S: {err}");
            }

            "SYN" => {
                let mut syn = Syn::new(self.pool.clone());
                Self::process_user_command(
                    self.protocol_version,
                    wr,
                    &mut self.authenticated_user,
                    &mut syn,
                    &command,
                )
                .await?;
            }

            "GCF" => {
                Self::process_command(self.protocol_version, wr, &mut Gcf, &command).await?;
            }

            "URL" => {
                Self::process_command(self.protocol_version, wr, &mut Url, &command).await?;
            }

            "CHG" => {
                let first_chg = self.authenticated_user.presence.is_none();
                let mut chg = Chg::new(self.broadcast_tx.clone(), first_chg);

                Self::process_user_command(
                    self.protocol_version,
                    wr,
                    &mut self.authenticated_user,
                    &mut chg,
                    &command,
                )
                .await?;
            }

            "UUX" => {
                let mut uux = Uux::new(self.broadcast_tx.clone());

                Self::process_user_command(
                    self.protocol_version,
                    wr,
                    &mut self.authenticated_user,
                    &mut uux,
                    &command,
                )
                .await?;
            }

            "PRP" => {
                let mut prp = Prp::new(self.pool.clone());
                Self::process_user_command(
                    self.protocol_version,
                    wr,
                    &mut self.authenticated_user,
                    &mut prp,
                    &command,
                )
                .await?;
            }

            "SBP" => {
                let mut sbp = Sbp::new(self.pool.clone());
                Self::process_user_command(
                    self.protocol_version,
                    wr,
                    &mut self.authenticated_user,
                    &mut sbp,
                    &command,
                )
                .await?;
            }

            "SDC" => {
                Self::process_command(self.protocol_version, wr, &mut Sdc, &command).await?;
            }

            "ADC" => {
                let mut adc = Adc::new(self.pool.clone(), self.broadcast_tx.clone());
                Self::process_user_command(
                    self.protocol_version,
                    wr,
                    &mut self.authenticated_user,
                    &mut adc,
                    &command,
                )
                .await?;
            }

            "REM" => {
                let mut rem = Rem::new(self.pool.clone(), self.broadcast_tx.clone());
                Self::process_user_command(
                    self.protocol_version,
                    wr,
                    &mut self.authenticated_user,
                    &mut rem,
                    &command,
                )
                .await?;
            }

            "ADG" => {
                let mut adg = Adg::new(self.pool.clone());
                Self::process_user_command(
                    self.protocol_version,
                    wr,
                    &mut self.authenticated_user,
                    &mut adg,
                    &command,
                )
                .await?;
            }

            "RMG" => {
                let mut rmg = Rmg::new(self.pool.clone());
                Self::process_user_command(
                    self.protocol_version,
                    wr,
                    &mut self.authenticated_user,
                    &mut rmg,
                    &command,
                )
                .await?;
            }

            "REG" => {
                let mut reg = Reg::new(self.pool.clone());
                Self::process_user_command(
                    self.protocol_version,
                    wr,
                    &mut self.authenticated_user,
                    &mut reg,
                    &command,
                )
                .await?;
            }

            "BLP" => {
                let mut blp = Blp::new(self.pool.clone());
                Self::process_user_command(
                    self.protocol_version,
                    wr,
                    &mut self.authenticated_user,
                    &mut blp,
                    &command,
                )
                .await?;
            }

            "GTC" => {
                let mut gtc = Gtc::new(self.pool.clone());
                Self::process_user_command(
                    self.protocol_version,
                    wr,
                    &mut self.authenticated_user,
                    &mut gtc,
                    &command,
                )
                .await?;
            }

            "XFR" => {
                let mut xfr = Xfr::new(self.broadcast_tx.clone());
                Self::process_user_command(
                    self.protocol_version,
                    wr,
                    &mut self.authenticated_user,
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

                trace!("S: {reply}");
            }

            "OUT" => return Err(ErrorCommand::Disconnect("Client disconnected".to_string())),

            _ => warn!("Unmatched command: {command}"),
        };

        Ok(())
    }
}
