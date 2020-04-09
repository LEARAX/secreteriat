use lazy_static::lazy_static;
use serde::Deserialize;
use serenity::client::Client;
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};
use serenity::model::{
    channel::{Message, ReactionType},
    gateway::Ready,
};
use serenity::prelude::{Context, EventHandler};
use serenity::utils::MessageBuilder;
use std::fs::File;
use std::io::prelude::*;

const REACT_SUCCESS: &str = "✅";
const REACT_FAIL: &str = "🟥";

lazy_static! {
    static ref CONFIG: Config = {
        let mut config_file = File::open("config.toml").expect("Failed to open config");
        let mut buffer = Vec::new();
        config_file
            .read_to_end(&mut buffer)
            .expect("Failed to read config");
        toml::from_slice(&buffer).expect("Failed to parse config")
    };
}

#[derive(Deserialize)]
struct Config {
    token: String,
    allowed_roles: toml::value::Array,
}

#[group]
#[commands(role)]
struct General;

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, _: Ready) {
        println!("Secreteriat online");
    }
}

fn main() {
    serenity::client::validate_token(&CONFIG.token).expect("Token does not appear valid");

    let mut client = Client::new(&CONFIG.token, Handler).expect("Error creating client");
    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.prefix(">"))
            .group(&GENERAL_GROUP),
    );

    if let Err(err) = client.start() {
        panic!("An error occurred while running the client: {:?}", err);
    }
}

#[command]
fn role(ctx: &mut Context, msg: &Message) -> CommandResult {
    let role_name = msg.content.split(" ").collect::<Vec<&str>>()[1];
    let mut member = msg.guild_id.unwrap().member(&ctx.http, msg.author.id)?;

    if let Some(arc) = msg.guild_id.unwrap().to_guild_cached(&ctx.cache) {
        if let Some(role) = arc.read().role_by_name(role_name) {
            if msg.member.as_ref().unwrap().roles.contains(&role.id) {
                println!("Removing role {} from user...", &role.name);
                let reaction = match member.remove_role(&ctx.http, role.id) {
                    Ok(_) => REACT_SUCCESS,
                    Err(_) => REACT_FAIL,
                };
                msg.react(&ctx.http, reaction)?;
            } else {
                println!("Adding role {} to user...", &role.name);
                let reaction = match member.add_role(&ctx.http, role.id) {
                    Ok(_) => REACT_SUCCESS,
                    Err(_) => REACT_FAIL,
                };
                msg.react(&ctx.http, reaction)?;
            }
        } else {
            msg.react(&ctx.http, REACT_FAIL)?;
        }
    } else {
        msg.react(&ctx.http, REACT_FAIL)?;
    }
    Ok(())
}
