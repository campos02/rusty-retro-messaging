use super::{broadcasted_command::BroadcastedCommand, command::Command};
use crate::models::transient::authenticated_user::AuthenticatedUser;

pub struct Chg;

impl Command for Chg {
    fn handle_with_authenticated_user(
        &mut self,
        command: &String,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, String> {
        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = args[1];

        // Validate presence
        match args[2] {
            "NLN" => (),
            "BSY" => (),
            "IDL" => (),
            "AWY" => (),
            "BRB" => (),
            "PHN" => (),
            "LUN" => (),
            "HDN" => (),
            _ => return Err(format!("201 {tr_id}\r\n")),
        }

        user.presence = Some(args[2].to_string());

        let Ok(client_id) = args[3].parse() else {
            return Err(format!("201 {tr_id}\r\n"));
        };

        user.client_id = Some(client_id);
        user.msn_object = if args.len() > 4 {
            Some(args[4].to_string())
        } else {
            None
        };

        Ok(vec![command.to_string()])
    }
}

impl BroadcastedCommand for Chg {
    fn convert(user: &AuthenticatedUser, command: &String) -> String {
        let mut args = command.trim().split(' ');
        args.next();
        args.next();

        let presence = args.next().expect("CHG to be converted has no presence");
        let client_id = args.next().expect("CHG to be converted has no client id");
        let mut msn_object = String::from("");

        if let Some(object) = args.next() {
            let mut object = String::from(object);
            object.insert_str(0, " ");
            msn_object = object;
        }

        let email = &user.email;
        let display_name = &user.display_name;
        format!("NLN {presence} {email} {display_name} {client_id}{msn_object}\r\n")
    }
}
