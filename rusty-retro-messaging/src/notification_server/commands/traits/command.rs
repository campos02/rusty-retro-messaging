pub trait Command {
    fn handle(&mut self, protocol_version: usize, command: &String) -> Result<Vec<String>, String>;
}
