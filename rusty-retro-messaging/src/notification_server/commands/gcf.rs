use super::command::Command;

pub struct Gcf;

impl Command for Gcf {
    fn handle(&mut self, protocol_version: usize, command: &String) -> Result<Vec<String>, String> {
        let _ = protocol_version;

        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = args[1];

        let mut payload = format!(r#"<?xml version= "1.0" encoding="utf-8" ?>"#);
        payload.push_str(
            format!(r#"<config><shield><cli maj="7" min="0" minbld="0" maxbld="9999" deny=" " />"#)
                .as_str(),
        );
        payload.push_str("</shield><block></block></config>");

        let length = payload.as_bytes().len();
        Ok(vec![format!(
            "GCF {tr_id} Shields.xml {length}\r\n{payload}"
        )])
    }
}
