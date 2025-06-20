use super::traits::user_command::UserCommand;
use crate::error_command::ErrorCommand;
use crate::models::group::Group;
use crate::models::transient::authenticated_user::AuthenticatedUser;
use crate::schema::groups::dsl::groups;
use crate::schema::groups::{guid, name, user_id};
use crate::schema::users::dsl::users;
use crate::{models::user::User, schema::users::email};
use diesel::insert_into;
use diesel::{
    BelongingToDsl, ExpressionMethods, MysqlConnection, QueryDsl, RunQueryDsl, SelectableHelper,
    r2d2::{ConnectionManager, Pool},
};

pub struct Adg {
    pool: Pool<ConnectionManager<MysqlConnection>>,
}

impl Adg {
    pub fn new(pool: Pool<ConnectionManager<MysqlConnection>>) -> Self {
        Adg { pool }
    }
}

impl UserCommand for Adg {
    fn handle(
        &self,
        protocol_version: usize,
        command: &str,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, ErrorCommand> {
        let _ = protocol_version;

        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = args[1];
        let group_name = args[2];

        let Ok(connection) = &mut self.pool.get() else {
            return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
        };

        let Ok(user_database) = users
            .filter(email.eq(&user.email))
            .select(User::as_select())
            .get_result(connection)
        else {
            return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
        };

        if Group::belonging_to(&user_database)
            .filter(name.eq(&group_name))
            .select(Group::as_select())
            .get_result(connection)
            .is_ok()
        {
            Err(ErrorCommand::Command(format!("228 {tr_id}\r\n")))
        } else {
            let group_guid = guid_create::GUID::rand().to_string().to_lowercase();

            if insert_into(groups)
                .values((
                    user_id.eq(&user_database.id),
                    name.eq(group_name),
                    guid.eq(&group_guid),
                ))
                .execute(connection)
                .is_err()
            {
                return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
            }

            Ok(vec![format!("ADG {tr_id} 1 {group_name} {group_guid}\r\n")])
        }
    }
}
