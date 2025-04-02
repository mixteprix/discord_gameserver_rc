use std::path::Path;
use std::process::Command;

use serenity::all::UserId;
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::{CommandOptionType, ResolvedOption, ResolvedValue};

use std::fs;

fn list_gameservers() -> String {
    // read folders
    // read Config file for name and stuff

    let mut list: Vec<String> = Vec::new();

    let entries = fs::read_dir("./").unwrap();
    for entry in entries {
        let entry = entry.unwrap(); // Handle the Result

        // Check if the entry is a directory
        if entry.file_type().unwrap().is_dir() {
            list.push(entry.path().display().to_string());
        }
    }

    // assemble the list to be printed
    let mut list_string = "The following gameservers are currently available:\n".to_string();
    for server in list {
        list_string.push_str("* ");
        list_string.push_str(server.as_str());
        list_string.push('\n');
    }

    if !list_string.is_empty() {
        list_string
    } else {
        "No gameservers are available at this time.".to_string()
    }
}

fn start_gameserver(gameserver: String) -> String {
    // execute the script
    // return Err("thing did not work") if it fails

    // spawning child may be unneccesary. consider changing in future
    let child = Command::new(format!("{gameserver}/start.sh")).spawn();

    match child {
        Ok(mut child_process) => "Started the gameserver".to_string(),
        Err(e) => "Failed to start the gameserver".to_string(),
    }
}

fn status_gameserver(gameserver: String) -> String {
    // run status script if available
    // return wheter or not available
    //

    if Path::new(&format!("gameservers/{gameserver}/status.sh")).exists() {
        let output = Command::new("bash")
            .arg(format!("gameservers/{gameserver}/status.sh"))
            .output()
            .expect("Failed to execute command");

        String::from_utf8(output.stdout).expect("output should be a String")
    } else {
        "Could not determine gameserver status".to_string()
    }
}

pub fn run(options: &[ResolvedOption], user: UserId) -> String {
    // check whitelist for this one

    let mut whitelist: Vec<String> = vec![];
    if let Ok(whitelist_file) = fs::read_to_string("gameservers/whitelist") {
        for line in whitelist_file.lines() {
            if !line.starts_with('#') {
                whitelist.push(line.to_string());
            }
        }
    } else {
        // not on whitelist obviously
        // maybe say something in log
    }

    if whitelist.contains(&user.to_string()) {
        dbg!(options);
        if let Some(ResolvedOption {
            value: ResolvedValue::SubCommand(command),
            ..
        }) = options.first()
        {
            if let Some(subcommand) = command.get(1) {
                println!("running {}", subcommand.name);
                match subcommand.name {
                    "list" => list_gameservers(),
                    "start" => {
                        if let Some(ResolvedOption {
                            value: ResolvedValue::String(gameserver),
                            ..
                        }) = options.get(2)
                        {
                            start_gameserver(gameserver.to_string())
                        } else {
                            "Ok, which one tho?".to_string()
                        }
                    }
                    "status" => {
                        if let Some(ResolvedOption {
                            value: ResolvedValue::String(gameserver),
                            ..
                        }) = options.get(2)
                        {
                            status_gameserver(gameserver.to_string())
                        } else {
                            "Ok, which one tho?".to_string()
                        }
                    }
                    _ => "please provide a valid command".to_string(),
                }
            } else {
                "Please use one of the available commands".to_string()
            }
        } else {
            "Please use one of the available commands".to_string()
        }
    } else {
        "You are not on the whitelist. Try asking a moderator or something.".to_string()
    }
}

pub fn register() -> CreateCommand {
    let subcommands = vec![
        CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "list",
            "Lists all available gameservers.",
        ),
        CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "start",
            "Starts a given gameserver.",
        )
        .add_sub_option(CreateCommandOption::new(
            CommandOptionType::String,
            "gameserver",
            "The gameserver you want to start.",
        )),
        CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "status",
            "Prints information on the currently active gameserver",
        )
        .add_sub_option(CreateCommandOption::new(
            CommandOptionType::String,
            "gameserver",
            "The gameserver you want to get the status of.",
        )),
    ];

    CreateCommand::new("gameserver")
        .description("control gameservers")
        .set_options(subcommands)
}
