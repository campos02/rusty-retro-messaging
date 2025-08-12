use crate::notification_server::handlers::process_command::{
    process_command, process_user_command,
};
use crate::{
    error_command::ErrorCommand,
    message::Message,
    models::transient::authenticated_user::AuthenticatedUser,
    notification_server::commands::{
        adc::Adc, adg::Adg, blp::Blp, chg::Chg, gcf::Gcf, gtc::Gtc, prp::Prp, reg::Reg, rem::Rem,
        rmg::Rmg, sbp::Sbp, sdc::Sdc, syn::Syn, url::Url, uux::Uux, xfr::Xfr,
    },
};
use log::{trace, warn};
use sqlx::{MySql, Pool};
use tokio::{io::AsyncWriteExt, net::tcp::WriteHalf, sync::broadcast};

pub async fn handle_user_command(
    protocol_version: usize,
    authenticated_user: &mut AuthenticatedUser,
    pool: &Pool<MySql>,
    broadcast_tx: &broadcast::Sender<Message>,
    wr: &mut WriteHalf<'_>,
    command: Vec<u8>,
) -> Result<(), ErrorCommand> {
    let command = str::from_utf8(&command).expect("Command contained invalid UTF-8");
    let args: Vec<&str> = command.trim().split(' ').collect();
    trace!("C: {command}");

    match *args.first().unwrap_or(&"") {
        "USR" => {
            let tr_id = *args.get(1).ok_or(ErrorCommand::Command("".to_string()))?;
            let err = format!("207 {tr_id}\r\n");

            wr.write_all(err.as_bytes())
                .await
                .expect("Could not send to client over socket");

            warn!("S: {err}");
        }

        "SYN" => {
            let syn = Syn::new(pool.clone());
            process_user_command(protocol_version, wr, authenticated_user, &syn, command).await?;
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
            process_user_command(protocol_version, wr, authenticated_user, &chg, command).await?;
        }

        "UUX" => {
            let uux = Uux::new(broadcast_tx.clone());
            process_user_command(protocol_version, wr, authenticated_user, &uux, command).await?;
        }

        "PRP" => {
            let prp = Prp::new(pool.clone());
            process_user_command(protocol_version, wr, authenticated_user, &prp, command).await?;
        }

        "SBP" => {
            let sbp = Sbp::new(pool.clone());
            process_user_command(protocol_version, wr, authenticated_user, &sbp, command).await?;
        }

        "SDC" => {
            process_command(protocol_version, wr, &Sdc, command).await?;
        }

        "ADC" => {
            let adc = Adc::new(pool.clone(), broadcast_tx.clone());
            process_user_command(protocol_version, wr, authenticated_user, &adc, command).await?;
        }

        "REM" => {
            let rem = Rem::new(pool.clone(), broadcast_tx.clone());
            process_user_command(protocol_version, wr, authenticated_user, &rem, command).await?;
        }

        "ADG" => {
            let adg = Adg::new(pool.clone());
            process_user_command(protocol_version, wr, authenticated_user, &adg, command).await?;
        }

        "RMG" => {
            let rmg = Rmg::new(pool.clone());
            process_user_command(protocol_version, wr, authenticated_user, &rmg, command).await?;
        }

        "REG" => {
            let reg = Reg::new(pool.clone());
            process_user_command(protocol_version, wr, authenticated_user, &reg, command).await?;
        }

        "BLP" => {
            let blp = Blp::new(pool.clone());
            process_user_command(protocol_version, wr, authenticated_user, &blp, command).await?;
        }

        "GTC" => {
            let gtc = Gtc::new(pool.clone());
            process_user_command(protocol_version, wr, authenticated_user, &gtc, command).await?;
        }

        "XFR" => {
            let xfr = Xfr::new(broadcast_tx.clone());
            process_user_command(protocol_version, wr, authenticated_user, &xfr, command).await?;
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
