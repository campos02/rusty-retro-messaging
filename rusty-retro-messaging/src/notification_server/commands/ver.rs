use super::traits::command::Command;

pub struct Ver;

impl Command for Ver {
    fn handle(&mut self, protocol_version: usize, command: &String) -> Result<Vec<String>, String> {
        let _ = protocol_version;

        let versions = vec!["MSNP12", "MSNP11"];
        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = args[1];

        for i in &args {
            for version in &versions {
                if *i == *version {
                    return Ok(vec![format!("VER {tr_id} {version}\r\n")]);
                }
            }
        }
        Err(format!("VER {tr_id} 0\r\n"))
    }
}
