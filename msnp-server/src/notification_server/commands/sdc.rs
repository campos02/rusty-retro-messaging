use super::command::Command;

pub struct Sdc;

impl Command for Sdc {
    fn handle(&mut self, command: &String) -> Result<Vec<String>, String> {
        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = args[1];

        Ok(vec![format!("SDC {tr_id} OK\r\n")])
    }
}
