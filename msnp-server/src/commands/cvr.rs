use super::command::Command;

pub struct Cvr;

impl Command for Cvr {
    fn handle(&mut self, command: &String) -> Result<Vec<String>, String> {
        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = args[1];

        Ok(vec![format!(
            "CVR {tr_id} 1.0.0000 1.0.0000 7.0.0425 http://download.microsoft.com/download/D/F/B/DFB59A5D-92DF-4405-9767-43E3DF10D25B/fr/Install_MSN_Messenger.exe http://messenger.msn.com/fr\r\n"
        )])
    }
}
