use crate::errors::command_error::CommandError;
use crate::errors::server_error::ServerError;
use crate::notification_server::commands::add::Add;
use crate::notification_server::commands::rea::Rea;
use crate::notification_server::handlers::process_command::{
    process_command, process_user_command,
};
use crate::{
    message::Message,
    models::transient::authenticated_user::AuthenticatedUser,
    notification_server::commands::{
        adc::Adc, adg::Adg, blp::Blp, chg::Chg, gcf::Gcf, gtc::Gtc, prp::Prp, reg::Reg, rem::Rem,
        rmg::Rmg, sbp::Sbp, sdc::Sdc, syn::Syn, url::Url, uux::Uux, xfr::Xfr,
    },
};
use log::{trace, warn};
use sqlx::{MySql, Pool};
use std::error;
use tokio::{io::AsyncWriteExt, net::tcp::WriteHalf, sync::broadcast};

pub async fn handle_user_command(
    protocol_version: u32,
    authenticated_user: &mut AuthenticatedUser,
    pool: &Pool<MySql>,
    broadcast_tx: &broadcast::Sender<Message>,
    wr: &mut WriteHalf<'_>,
    version_number: &mut u32,
    command: Vec<u8>,
) -> Result<(), Box<dyn error::Error + Send + Sync>> {
    let command = str::from_utf8(&command)?;
    let args: Vec<&str> = command.trim().split(' ').collect();
    trace!("C: {command}");

    match *args.first().unwrap_or(&"") {
        "USR" => {
            let tr_id = *args.get(1).ok_or(CommandError::NoTrId)?;
            let err = format!("207 {tr_id}\r\n");

            wr.write_all(err.as_bytes()).await?;
            warn!("S: {err}");
        }

        "SYN" => {
            let syn = Syn::new(pool.clone());
            process_user_command(
                protocol_version,
                wr,
                authenticated_user,
                version_number,
                &syn,
                command,
            )
            .await?;
        }

        "GCF" => {
            process_command(protocol_version, wr, &Gcf, command).await?;
        }

        "URL" => {
            process_command(protocol_version, wr, &Url, command).await?;
        }

        "CHG" => {
            let first_chg = authenticated_user.presence.is_none();
            let chg = Chg::new(broadcast_tx.clone(), first_chg);
            process_user_command(
                protocol_version,
                wr,
                authenticated_user,
                version_number,
                &chg,
                command,
            )
            .await?;
        }

        "UUX" => {
            let uux = Uux::new(broadcast_tx.clone());
            process_user_command(
                protocol_version,
                wr,
                authenticated_user,
                version_number,
                &uux,
                command,
            )
            .await?;
        }

        "PRP" => {
            let prp = Prp::new(pool.clone());
            process_user_command(
                protocol_version,
                wr,
                authenticated_user,
                version_number,
                &prp,
                command,
            )
            .await?;
        }

        "SBP" => {
            let sbp = Sbp::new(pool.clone());
            process_user_command(
                protocol_version,
                wr,
                authenticated_user,
                version_number,
                &sbp,
                command,
            )
            .await?;
        }

        "SDC" => {
            process_command(protocol_version, wr, &Sdc, command).await?;
        }

        "ADC" => {
            let adc = Adc::new(pool.clone(), broadcast_tx.clone());
            process_user_command(
                protocol_version,
                wr,
                authenticated_user,
                version_number,
                &adc,
                command,
            )
            .await?;
        }

        "ADD" => {
            let add = Add::new(pool.clone(), broadcast_tx.clone());
            process_user_command(
                protocol_version,
                wr,
                authenticated_user,
                version_number,
                &add,
                command,
            )
            .await?;
        }

        "REM" => {
            let rem = Rem::new(pool.clone(), broadcast_tx.clone());
            process_user_command(
                protocol_version,
                wr,
                authenticated_user,
                version_number,
                &rem,
                command,
            )
            .await?;
        }

        "ADG" => {
            let adg = Adg::new(pool.clone());
            process_user_command(
                protocol_version,
                wr,
                authenticated_user,
                version_number,
                &adg,
                command,
            )
            .await?;
        }

        "RMG" => {
            let rmg = Rmg::new(pool.clone());
            process_user_command(
                protocol_version,
                wr,
                authenticated_user,
                version_number,
                &rmg,
                command,
            )
            .await?;
        }

        "REG" => {
            let reg = Reg::new(pool.clone());
            process_user_command(
                protocol_version,
                wr,
                authenticated_user,
                version_number,
                &reg,
                command,
            )
            .await?;
        }

        "REA" => {
            let rea = Rea::new(pool.clone());
            process_user_command(
                protocol_version,
                wr,
                authenticated_user,
                version_number,
                &rea,
                command,
            )
            .await?;
        }

        "BLP" => {
            let blp = Blp::new(pool.clone());
            process_user_command(
                protocol_version,
                wr,
                authenticated_user,
                version_number,
                &blp,
                command,
            )
            .await?;
        }

        "GTC" => {
            let gtc = Gtc::new(pool.clone());
            process_user_command(
                protocol_version,
                wr,
                authenticated_user,
                version_number,
                &gtc,
                command,
            )
            .await?;
        }

        "XFR" => {
            let xfr = Xfr::new(broadcast_tx.clone());
            process_user_command(
                protocol_version,
                wr,
                authenticated_user,
                version_number,
                &xfr,
                command,
            )
            .await?;
        }

        "PNG" => {
            let reply = if protocol_version >= 9 {
                "QNG 60\r\n"
            } else {
                "QNG\r\n"
            };

            wr.write_all(reply.as_bytes()).await?;
            trace!("S: {reply}");
        }

        "OUT" => {
            return Err(ServerError::Disconnected.into());
        }

        _ => warn!("Unmatched command: {command}"),
    };

    Ok(())
}
