use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::{CommandOptionType, ResolvedOption, ResolvedValue};

pub fn run(options: &[ResolvedOption]) -> String {
    // if let Some(ResolvedOption {

    // open list of servers
    // check if exists
        // tell user if not
    // start server
        // tell user if failed


    return "TODO".to_owned()
}

pub fn register() -> CreateCommand {
    CreateCommand::new("start").description("Start a selected gameserver.").add_option(
        CreateCommandOption::new(CommandOptionType::User, "server", "the server to start (see /list)")
            .required(true),
    )
}