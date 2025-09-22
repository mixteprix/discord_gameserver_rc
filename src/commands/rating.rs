use core::num;
use std::collections::{hash_map, HashMap};
use std::convert::TryInto;
use std::fs::{self, File};
use std::hash::Hash;
use std::io::{Error, Write};
use std::u32;

use serde::{self, Deserialize, Serialize};
use serenity::all::{
    CommandInteraction, CommandOption, CommandType, ErrorResponse, GetMessages, GuildId, Message, MessageBuilder, MessageId, MessageReaction, Reaction, ReactionType, Timestamp, User
};
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::futures::channel;
use serenity::model::application::{CommandOptionType, ResolvedOption, ResolvedValue};
use serenity::prelude::*;
use tabled::{derive, Table, Tabled, settings::Style};
use tokio::fs::create_dir_all;

pub trait MessageStuff {
    fn is_eligible(&self) -> bool;
}

impl MessageStuff for Message {
    fn is_eligible(&self) -> bool {
        // attachments/ embeds dont't seem to be visible to bots.
        // self.attachments
        //     .first()
        //     .and_then(|attachment| attachment.content_type.as_ref())
        //     .map_or(false, |content_type| content_type.starts_with("image") && !self.author.bot)

        !self.reactions.is_empty()
    }
}

#[derive(Tabled)]
struct RatingEntity {
    name: String,
    avg: f32,
    std: f32,
    // median: f32,
    // todo: more
    total_posts: u32,
}

#[derive(Serialize, Deserialize, Clone)]
struct MessageReactionShort {
    count: u64,
    reaction_type: ReactionType
}

impl From<MessageReaction> for MessageReactionShort {
    fn from(value: MessageReaction) -> Self {
        MessageReactionShort {
            count: value.count,
            reaction_type: value.reaction_type,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct RatedPost {
    id: MessageId,
    author: User,
    reactions: Vec<MessageReactionShort>,
    timestamp: Timestamp,
}

impl PartialEq for RatedPost {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

fn merge_cache_and_new(old: &Vec<RatedPost>, new: &Vec<RatedPost>) -> Vec<RatedPost> {
    // todo: maybe optimise: old should usually contain more entries than new, so cloning old and writing new into it when appropriate may be preferable.
    let mut merged_posts: Vec<RatedPost> = new.clone();

    for post in old {
        if !new.contains(&post) {
            merged_posts.push(post.clone());
        }
    }

    merged_posts
}

fn get_path(guild_id: GuildId, channel_id: serenity::all::ChannelId) -> String {
    let path = format!(
        "./cache/{}/{}/rated_posts.json",
        guild_id.to_string(),
        channel_id.to_string()
    );
    println!("{}", path);
    path
}

async fn get_messages(
    ctx: &Context,
    channel_id: serenity::all::ChannelId,
    number_to_update: u64,
) -> Option<Vec<serenity::all::Message>> {
    // todo: make this variable by input
    let mut limit: u64 = number_to_update;

    let mut messages: Vec<serenity::all::Message> = channel_id
        .messages(ctx, GetMessages::new().limit(100))
        .await
        .expect("could not get messages");
    if messages.is_empty() {
        println!("no messages retrieved (empty).");
        return None;
    }
    println!("{messages:?}");
    let mut last_message_id = messages.last().expect("failed to get last message id.").id;

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
        last_message_id = channel_messages
            .last()
            .expect("failed to get last message id.")
            .id;
        messages.append(&mut channel_messages);
        limit -= 1;
    }

    if messages.is_empty() {
        println!("messages empty.");
        return Option::None;
    }
    Option::Some(messages)
}

async fn update_cache(
    ctx: &Context,
    channel_id: serenity::all::ChannelId,
    guild_id: GuildId,
    number_to_update: u64,
) -> Result<(), String> {
    if let Some(messages) = get_messages(ctx, channel_id, number_to_update).await {
        let mut rated_posts: Vec<RatedPost> = vec![];

        for msg in messages.clone() {
            if msg.is_eligible() {
                rated_posts.push(RatedPost {
                    id: msg.id,
                    author: msg.author,
                    reactions: msg.reactions.iter().map(|r| MessageReactionShort::from(r.to_owned())).collect(),
                    timestamp: msg.timestamp,
                });
            }
        }

        if rated_posts.is_empty() {
            print!("no eligible messages retrieved.");
            return Err("No eligible messages retrieved.".to_owned());
        }

        // todo: probably this elsewhere
        // let cachedata = serde_json::ser::to_string_pretty(&rated_posts).unwrap();

        let cache_file_path: String = get_path(guild_id, channel_id);

        if fs::exists(&cache_file_path).expect("existence of cache file could not be determined.") {
            println!("cache file at {} exists.", &cache_file_path);
            let cached_data_old: Vec<RatedPost> = serde_json::from_str(
                &fs::read_to_string(&cache_file_path)
                    .expect(&format!("could not read file at {cache_file_path}")),
            )
            .expect("could not deserialize cached file.");

            merge_cache_and_new(&cached_data_old, &rated_posts);

            let cachedata = serde_json::ser::to_string_pretty(&rated_posts)
                .expect("failed to serialize data to cache. (file found)");

            fs::write(&cache_file_path, cachedata).expect("faled to write to cache. (file found)");
        } else {
            println!("no cache file at {}, creating a new one.", &cache_file_path);

            create_dir_all(
                std::path::Path::new(&cache_file_path)
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new("")),
            )
            .await
            .expect("failed to create parent dir for cache.");

            let cachedata = serde_json::ser::to_string_pretty(&rated_posts)
                .expect("failed to serialize data to cache. (no file found)");
            fs::write(&cache_file_path, cachedata)
                .expect("failed to write to cache. (no file found)");
        }

        return Ok(());
    } else {
        println!("no messages retrieved.");
        return Err("No messages retrieved.".to_owned());
    }
}

async fn get_rated_posts_from_cache(
    guild_id: GuildId,
    channel_id: serenity::all::ChannelId,
) -> Option<Vec<RatedPost>> {
    let cache_path = get_path(guild_id, channel_id);

    serde_json::from_str(&fs::read_to_string(cache_path).unwrap()).unwrap()
}

fn get_scores(messages: &Vec<RatedPost>) -> Option<std::collections::HashMap<&str, Vec<Vec<u64>>>> {
    // iterate over msg
    // users into lookup table
    // append score to vector
    // append in cronological order?
    let mut reaction_data: std::collections::HashMap<&str, Vec<Vec<u64>>> =
        std::collections::HashMap::new();

    // make a vector of all scores given to each user
    for message in messages {
        let mut message_reaction_data: Vec<u64> = vec![];
        for reaction in &message.reactions {
            // if reaction is a rating
            if let ReactionType::Unicode(unicode) = &reaction.reaction_type {
                let num_reacts = reaction.count;
                match unicode.as_str() {
                    "0️⃣" => {
                        for _ in 0..num_reacts {
                            message_reaction_data.push(0);
                        }
                    }
                    "1️⃣" => {
                        for _ in 0..num_reacts {
                            message_reaction_data.push(1);
                        }
                    }
                    "2️⃣" => {
                        for _ in 0..num_reacts {
                            message_reaction_data.push(2);
                        }
                    }
                    "3️⃣" => {
                        for _ in 0..num_reacts {
                            message_reaction_data.push(3);
                        }
                    }
                    "4️⃣" => {
                        for _ in 0..num_reacts {
                            message_reaction_data.push(4);
                        }
                    }
                    "5️⃣" => {
                        for _ in 0..num_reacts {
                            message_reaction_data.push(5);
                        }
                    }
                    "6️⃣" => {
                        for _ in 0..num_reacts {
                            message_reaction_data.push(6);
                        }
                    }
                    "7️⃣" => {
                        for _ in 0..num_reacts {
                            message_reaction_data.push(7);
                        }
                    }
                    "8️⃣" => {
                        for _ in 0..num_reacts {
                            message_reaction_data.push(8);
                        }
                    }
                    "9️⃣" => {
                        for _ in 0..num_reacts {
                            message_reaction_data.push(9);
                        }
                    }
                    "🔟" => {
                        for _ in 0..num_reacts {
                            message_reaction_data.push(10);
                        }
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

pub async fn run(
    options: &[ResolvedOption<'_>],
    ctx: &Context,
    command: &CommandInteraction,
) -> String {
    let channel_id = command.channel_id;

    let mut number_of_msg_to_fetch = 2;

    dbg!(options);
    if let Some(ResolvedOption {
        name,
        value: ResolvedValue::SubCommand(command),
        ..
    }) = options.first()
    {
        // subcommand has further input
        if let Some(subcommand) = command.first() {
            println!("running {}", subcommand.name);
            dbg!(subcommand);
            dbg!(name);
            match name.to_owned() {
                "rating" => {
                    if let ResolvedValue::Integer(option) = subcommand.value {
                        if option < 0 {
                            return "Invalid input for number of messages(blocks) to update. (must be positive)".to_owned();
                        } else {
                            number_of_msg_to_fetch = option;
                        }
                    } else {
                        return "Invalid input for number of messages(blocks) to update.".to_owned();
                    }
                }
                _ => {}
            }
        }
    };


    if let Ok(_) = update_cache(
        ctx,
        channel_id,
        command.guild_id.unwrap(),
        (number_of_msg_to_fetch-1)
            .try_into()
            .expect("i64 input could not be converted to u64"),
    )
    .await
    {
    } else {
        println!("failed to get new messages.");
        return "failed to get new messages".to_string();
    }

    let messages = get_rated_posts_from_cache(command.guild_id.unwrap(), channel_id)
        .await
        .unwrap();

    if let Some(reaction_data) = get_scores(&messages) {
        // todo: consider making this a vector
        let mut answer = format!("# Average Scores(last {}): \n```\n", number_of_msg_to_fetch*100);

        let mut data: Vec<RatingEntity> = vec![];

        for (user, scores) in reaction_data {
            let num_posts = scores.clone().len();

            // get the averages of all the posts
            let averages: Vec<f32> = scores
                .iter()
                .map(|x| x.iter().map(|&x| x as f32).sum::<f32>() / x.len() as f32)
                .collect();

            // get the average of post scores
            let sum: f32 = averages.iter().sum();
            let avg: f32 = sum as f32 / num_posts as f32;

            // get the standard deviation of post scores
            let mut sn_sum: f32 = 0.0;
            averages
                .iter()
                .for_each(|x| sn_sum += (x - avg) * (x - avg));
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
                data.push(RatingEntity {
                    name: user.to_owned(),
                    avg: avg,
                    std: sn,
                    total_posts: num_posts as u32,
                });
            }
        }
        data.sort_by(|a, b| a.avg.partial_cmp(&b.avg).unwrap());

        let mut table = Table::new(data);
        table.with(Style::markdown());

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
    let subcommands = vec![
        CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "rating",
            "Rates posts and returns a table.",
        )
        .add_sub_option(CreateCommandOption::new(
            CommandOptionType::Integer,
            "update",
            "The number of posts (×100) to update.",
        )),

    ];

    CreateCommand::new("rating")
        .description("Display the average ratings per user in this channel.")
        .set_options(subcommands)

    
}
