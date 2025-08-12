use super::traits::command::Command;
use crate::error_command::ErrorCommand;

pub struct Gcf;

impl Command for Gcf {
    async fn handle(
        &self,
        protocol_version: usize,
        command: &str,
    ) -> Result<Vec<String>, ErrorCommand> {
        let _ = protocol_version;

        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = *args.get(1).ok_or(ErrorCommand::Command("".to_string()))?;

        let mut payload = r#"<?xml version= "1.0" encoding="utf-8" ?>"#.to_string();
        payload.push_str(
            r#"<config><shield><cli maj="7" min="0" minbld="0" maxbld="9999" deny=" " />"#,
        );
        payload.push_str("</shield><block></block></config>");

        let length = payload.len();
        Ok(vec![format!(
            "GCF {tr_id} Shields.xml {length}\r\n{payload}"
        )])
    }
}
