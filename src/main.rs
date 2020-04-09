use lazy_static::lazy_static;
use serde::Deserialize;
use serenity::framework::standard::{
    macros::{command, group},
    Args, CommandResult, Delimiter, StandardFramework,
};
use serenity::model::{channel::Message, gateway::Ready, user::CurrentUser};
use serenity::prelude::*;
use serenity::utils::Color;
use std::fs::File;
use std::io::prelude::*;

const COMMAND_LIST: &[Command<'static>] = &[
    Command {
        name: "help",
        description: "Prints help information",
    },
    Command {
        name: "role",
        description: "Toggles a whitelisted role",
    },
    Command {
        name: "roles",
        description: "Prints a list of roles",
    },
];

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

struct Command<'a> {
    name: &'a str,
    description: &'a str,
}

#[derive(Deserialize)]
struct Config {
    token: String,
    allowed_roles: std::collections::BTreeMap<String, String>,
}

#[group]
#[commands(help, role, roles)]
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
            .configure(|c| {
                c.prefix(">");
                c.allow_dm(false)
            })
            .group(&GENERAL_GROUP)
            .before(|ctx, msg, _| {
                if let Some(channel) = msg.channel_id.name(&ctx.cache) {
                    channel == "roles"
                } else {
                    false
                }
            })
            .on_dispatch_error(|ctx, msg, err| {
                println!("UNHANDLED ERROR: {:?}", err);
                msg.react(&ctx.http, REACT_FAIL).unwrap();
            }),
    );

    if let Err(err) = client.start() {
        panic!("An error occurred while running the client: {:?}", err);
    }
}

#[command]
fn help(ctx: &mut Context, msg: &Message) -> CommandResult {
    println!("Printing help message...");
    msg.channel_id.send_message(&ctx.http, |response| {
        response.embed(|embed| {
            let e = embed.title("Command list")
                .description("All commands must be prefixed by `>`")
                .author(|a| {
                    a.name("The Secreteriat")
                    .icon_url(CurrentUser::face(&ctx.http.get_current_user().unwrap()))
                })
                .color(Color::from_rgb(127,127,255))
                .thumbnail("https://upload.wikimedia.org/wikipedia/commons/thumb/2/2f/Flag_of_the_United_Nations.svg/1280px-Flag_of_the_United_Nations.svg.png");
            for command in COMMAND_LIST.iter() {
                e.field(command.name, command.description, true);
            };
            e
        });
        response
    })?;
    Ok(())
}

#[command]
fn role(ctx: &mut Context, msg: &Message) -> CommandResult {
    let msg_split = msg.content.split(" ").collect::<Vec<&str>>();
    if msg_split.len() >= 2 {
        let role_name = &msg.content[6..];
        let mut member = msg.guild_id.unwrap().member(&ctx.http, msg.author.id)?;

        if CONFIG.allowed_roles.contains_key(&String::from(role_name)) {
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
        } else {
            msg.react(&ctx.http, REACT_FAIL)?;
        }
    } else {
        msg.react(&ctx.http, REACT_FAIL)?;
        help(ctx, msg, Args::new(&msg.content, &[Delimiter::Single(' ')]))?;
    }
    Ok(())
}

#[command]
fn roles(ctx: &mut Context, msg: &Message) -> CommandResult {
    println!("Printing role list...");
    msg.channel_id.send_message(&ctx.http, |response| {
        response.embed(|embed| {
            let e = embed.title("Role list")
                .author(|a| {
                    a.name("The Secreteriat")
                    .icon_url(CurrentUser::face(&ctx.http.get_current_user().unwrap()))
                })
                .color(Color::from_rgb(127,127,255))
                .thumbnail("https://upload.wikimedia.org/wikipedia/commons/thumb/2/2f/Flag_of_the_United_Nations.svg/1280px-Flag_of_the_United_Nations.svg.png");
            for role in &CONFIG.allowed_roles {
                e.field(role.0, role.1, true);
            };
            e
        });
        response
    })?;
    Ok(())
}
