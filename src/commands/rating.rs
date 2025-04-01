use std::collections::{hash_map, HashMap};
use std::hash::Hash;
use std::u32;

use serenity::all::{
    CommandInteraction, GetMessages, Message, MessageBuilder, Reaction, ReactionType,
};
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::futures::channel;
use serenity::model::application::{CommandOptionType, ResolvedOption, ResolvedValue};
use serenity::prelude::*;

async fn get_messages(
    ctx: &Context,
    channel_id: serenity::all::ChannelId,
) -> Option<Vec<serenity::all::Message>> {
    let mut messages: Vec<serenity::all::Message> = channel_id
        .messages(ctx, GetMessages::new().limit(100))
        .await
        .expect("could not get messages");
    if messages.is_empty() {
        return None;
    }
    println!("{messages:?}");
    let mut last_message_id = messages.last().unwrap().id;

    loop {
        // Fetch messages in batches of 100
        println!("😐getting messages");
        println!("{channel_id:?}");
        let mut channel_messages = channel_id
            .messages(ctx, GetMessages::new().before(last_message_id).limit(100))
            .await
            .expect("could not get message");

        if channel_messages.is_empty() {
            break; // No more messages to fetch, break the loop
        }
        // println!("{channel_messages:?}");

        // Set the last message ID for pagination
        last_message_id = channel_messages.last().unwrap().id;
        messages.append(&mut channel_messages);
    }

    if messages.is_empty() {
        return Option::None;
    }
    Option::Some(messages)
}

fn get_scores(messages: &Vec<Message>) -> Option<std::collections::HashMap<&str, Vec<u8>>> {
    // iterate over msg
    // users into lookup table
    // append score to vector
    // append in cronological order?
    let mut reaction_data: std::collections::HashMap<&str, Vec<u8>> =
        std::collections::HashMap::new();

    // make a vector of all scores given to each user
    for message in messages {
        for reaction in &message.reactions {
            if let ReactionType::Unicode(unicode) = &reaction.reaction_type {
                if unicode.starts_with(&['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']) { // todo: do exact match, rather than this nonsense.
                    let score = reaction_data
                        .entry(message.author.name.as_str())
                        .or_insert(vec![unicode.as_bytes()[0] - 48]);
                    score.push(unicode.as_bytes()[0] - 48);
                } else if unicode == "🔟" {
                    let score = reaction_data
                        .entry(message.author.name.as_str())
                        .or_insert(vec![10]);
                    score.push(10);
                }
            }
        }
    }

    if reaction_data.is_empty() {
        return None;
    } else {
        Some(reaction_data)
    }
}

pub async fn run(ctx: &Context, command: &CommandInteraction) -> String {
    let channel_id = command.channel_id;

    let messages = get_messages(ctx, channel_id)
        .await
        .expect("did not find any messages");

    if let Some(reaction_data) = get_scores(&messages) {
        let mut answer = "# Average Scores: \n".to_string();
        for (user, scores) in reaction_data {
            let sum: u32 = scores.iter().map(|&x| x as u32).sum();
            let avg: f32 = sum as f32 / scores.len() as f32;
            answer += format!("{user}: {avg}\n").as_str();
        }
        return answer;
    } else {
        return "No scores have been given in this channel. \n Try rating some post using the 1️⃣...🔟 emojis as reactions.".to_owned();
    }

    // todo: arguments for avg | trend | median | deviation | stuff maybe
    // print average scores
    // for (key, value) in &scores {
    //     println!("{key}: {value}");
    // }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("rating")
        .description("Display the average ratings per user in this channel.")
}
