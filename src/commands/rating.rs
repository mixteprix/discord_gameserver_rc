use core::num;
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
use tabled::{Table, Tabled};


#[derive(Tabled)]
struct RatingEntity {
    name: String,
    avg: f32,
    std: f32,
    // median: f32,
    // todo: more
    total_posts: u32,
}


async fn get_messages(
    ctx: &Context,
    channel_id: serenity::all::ChannelId,
) -> Option<Vec<serenity::all::Message>> {

    // todo: make this variable by input
    let mut limit: u32 = 5;

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

        if limit == 0 {
            break; // limit, so discord does not block me
        }
        // println!("{channel_messages:?}");

        // Set the last message ID for pagination
        last_message_id = channel_messages.last().unwrap().id;
        messages.append(&mut channel_messages);
        limit -= 1;
    }

    if messages.is_empty() {
        return Option::None;
    }
    Option::Some(messages)
}


fn get_scores(messages: &Vec<Message>) -> Option<std::collections::HashMap<&str, Vec<Vec<u8>>>> {
    // iterate over msg
    // users into lookup table
    // append score to vector
    // append in cronological order?
    let mut reaction_data: std::collections::HashMap<&str, Vec<Vec<u8>>> =
        std::collections::HashMap::new();

    // make a vector of all scores given to each user
    for message in messages {
        let mut message_reaction_data: Vec<u8> = vec![];
        for reaction in &message.reactions {
            // if reaction is a rating
            if let ReactionType::Unicode(unicode) = &reaction.reaction_type {
                match unicode.as_str() {
                    "0️⃣" => {
                        message_reaction_data.push(0);
                    }
                    "1️⃣" => {
                        message_reaction_data.push(1);
                    }
                    "2️⃣" => {
                        message_reaction_data.push(2);
                    }
                    "3️⃣" => {
                        message_reaction_data.push(3);
                    }
                    "4️⃣" => {
                        message_reaction_data.push(4);
                    }
                    "5️⃣" => {
                        message_reaction_data.push(5);
                    }
                    "6️⃣" => {
                        message_reaction_data.push(6);
                    }
                    "7️⃣" => {
                        message_reaction_data.push(7);
                    }
                    "8️⃣" => {
                        message_reaction_data.push(8);
                    }
                    "9️⃣" => {
                        message_reaction_data.push(9);
                    }
                    "🔟" => {
                        message_reaction_data.push(10);
                    }
                    _ => {
                        // not a rating
                    }
                }
            }
        }

        if !message_reaction_data.is_empty() {
            reaction_data
                .entry(message.author.name.as_str())
                .or_insert(vec![])
                .push(message_reaction_data)
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
        // todo: consider making this a vector
        let mut answer = "# Average Scores(of the last 500 messages): \n```\n".to_string();

        let mut data: Vec<RatingEntity> = vec![];

        for (user, scores) in reaction_data {

            let num_posts = scores.clone().len();

            // get the averages of all the posts
            let averages: Vec<f32> = scores
                .iter()
                .map(|x| x.iter().map(|&x| x as f32).sum::<f32>() / x.len() as f32).collect();

            // get the average of post scores
            let sum: f32 = averages.iter().sum();
            let avg: f32 = sum as f32 / num_posts as f32;

            // get the standard deviation of post scores
            let mut sn_sum: f32 = 0.0;
            averages.iter().for_each(|x| sn_sum += (x - avg)*(x - avg));
            let sn = (sn_sum / num_posts as f32).sqrt();

            for x in &scores {
                println!("---");
                println!("{:?}", x);
                println!("---")
            }

            println!("{user}: avg={avg}, sum={sum}, total={}", num_posts);

            // ignore all posters, who have posted less than 3 meals
            if num_posts >= 3 {
                // answer += format!("{user}: {avg}\n").as_str();
                data.push(RatingEntity { name: user.to_owned(), avg: avg, std: sn, total_posts: num_posts as u32 });
            }
        }
        data.sort_by(|a, b| a.avg.partial_cmp(&b.avg).unwrap());

        let table = Table::new(data);

        answer += &table.to_string().clone();
        answer += "```\n";

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
