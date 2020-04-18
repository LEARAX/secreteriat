use lazy_static::lazy_static;
use serde::Deserialize;
use serenity::framework::standard::{
    macros::{command, group},
    Args, CommandResult, Delimiter, StandardFramework,
};
use serenity::model::{channel::Message, gateway::Ready, user::CurrentUser};
use serenity::prelude::*;
use serenity::utils::Color;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;

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
    name: String,
    thumbnail: String,
    bot_channel: String,
    owners: Vec<String>,
    public_roles: BTreeMap<String, String>,
}

#[group]
#[owners_only]
#[commands(rquery, rrename, rverify)]
struct Debug;

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
            .configure(|config| {
                let c = config.prefix(">").allow_dm(false).case_insensitivity(true);
                if !CONFIG.owners.is_empty() {
                    let mut processed_owners = vec![];
                    for owner in CONFIG.owners.iter() {
                        println!("Adding owner with ID {}", owner);
                        processed_owners.push(serenity::model::id::UserId(
                            u64::from_str(owner).expect("Failed to parse owner user ID"),
                        ))
                    }
                    c.owners(processed_owners.into_iter().collect());
                }
                c
            })
            .group(&GENERAL_GROUP)
            .group(&DEBUG_GROUP)
            .before(|ctx, msg, _| {
                if let Some(channel) = msg.channel_id.name(&ctx.cache) {
                    channel == CONFIG.bot_channel
                } else {
                    false
                }
            })
            .on_dispatch_error(|ctx, msg, err| {
                eprintln!("ENCOUNTERED UNHANDLED ERROR: {:?}", err);
                match msg.react(&ctx.http, REACT_FAIL) {
                    Ok(_) => {}
                    Err(rerr) => eprintln!("Encountered additional reaction error: {:?}", rerr),
                }
            }),
    );

    if let Err(err) = client.start() {
        panic!("An error occurred while running the client: {:?}", err);
    }
}

fn role_search<'a>(
    name: &'a str,
    role_tree: &'a BTreeMap<String, String>,
) -> Option<(f32, &'a String)> {
    let mut max_similarity_pair = None;
    println!("Searching for similar roles...");
    for key in role_tree.keys() {
        let similarity = trigram::similarity(key, name);
        println!("Key {} has similarity: {}", key, similarity);
        if let Some((max_similarity, _)) = max_similarity_pair {
            if similarity > max_similarity {
                max_similarity_pair = Some((similarity, key));
            }
        } else if similarity > 0.0 {
            max_similarity_pair = Some((similarity, key));
        }
    }
    max_similarity_pair
}

#[command]
fn help(ctx: &mut Context, msg: &Message) -> CommandResult {
    println!("Printing help message...");
    msg.channel_id.send_message(&ctx.http, |response| {
        response.embed(|embed| {
            let e = embed
                .title("Commands")
                .description("All commands must be prefixed by `>`")
                .author(|a| {
                    a.name(&CONFIG.name)
                        .icon_url(CurrentUser::face(&ctx.http.get_current_user().unwrap()))
                })
                .color(Color::FADED_PURPLE)
                .thumbnail(&CONFIG.thumbnail);
            for command in COMMAND_LIST.iter() {
                e.field(command.name, command.description, true);
            }
            e
        });
        response
    })?;
    Ok(())
}

#[command]
fn role(ctx: &mut Context, msg: &Message) -> CommandResult {
    println!("Role command called");
    if msg.content.len() > 6 {
        if let Some(guild_id) = msg.guild_id {
            if let Some(arc) = guild_id.to_guild_cached(&ctx.cache) {
                let mut member = arc.read().member(&ctx.http, msg.author.id)?;
                if let Some(matched_role) = role_search(&msg.content[6..], &CONFIG.public_roles) {
                    println!(
                        "Matched role {} with similarity {}",
                        matched_role.1, matched_role.0
                    );
                    if let Some(role) = arc.read().role_by_name(matched_role.1) {
                        println!("Found role");
                        if member.roles.contains(&role.id) {
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
                        eprintln!("Failed to find role");
                        msg.react(&ctx.http, REACT_FAIL)?;
                    }
                } else {
                    eprintln!("Similarity search found no results");
                    msg.react(&ctx.http, REACT_FAIL)?;
                }
            } else {
                eprintln!("Failed to cache guild");
                msg.react(&ctx.http, REACT_FAIL)?;
            }
        } else {
            eprintln!("Failed to grab guild");
            msg.react(&ctx.http, REACT_FAIL)?;
        }
    } else {
        msg.react(&ctx.http, REACT_FAIL)?;
        help(ctx, msg, Args::new(&msg.content, &[Delimiter::Single(' ')]))?;
    }
    Ok(())
}

#[command]
fn rquery(ctx: &mut Context, msg: &Message) -> CommandResult {
    println!("DEBUG RQUERY REQUEST");
    if let Some(guild_id) = msg.guild_id {
        if let Some(guild) = guild_id.to_guild_cached(&ctx.cache) {
            msg.channel_id.send_message(&ctx.http, |response| {
                println!("Sending RQUERY response...");
                response.embed(|embed| {
                    let e = embed
                        .title("Role list")
                        .author(|a| {
                            a.name(&CONFIG.name)
                                .icon_url(CurrentUser::face(&ctx.http.get_current_user().unwrap()))
                        })
                        .color(Color::BLUE)
                        .thumbnail(&CONFIG.thumbnail);
                    for (role_id, role) in guild.read().roles.iter() {
                        e.field(&role.name, role_id, false);
                    }
                    e
                });
                response
            })?;
        } else {
            eprintln!("Failed to cache guild");
            msg.react(&ctx.http, REACT_FAIL)?;
        }
    } else {
        eprintln!("Failed to grab guild");
        msg.react(&ctx.http, REACT_FAIL)?;
    }
    Ok(())
}

#[command]
fn rrename(ctx: &mut Context, msg: &Message) -> CommandResult {
    println!("DEBUG ROLE RENAME REQUEST");
    if msg.content.len() > 28 {
        if let Ok(old_id) = msg.content[9..27].parse::<u64>() {
            println!("Old ID: {}", old_id);
            let new_name = &msg.content[28..];
            println!("Requested name: {}", new_name);
            if let Some(guild_id) = msg.guild_id {
                if let Some(guild) = guild_id.to_guild_cached(&ctx.cache) {
                    let guild = guild.read();
                    let mut role_wrap = None;
                    for (role_id, role) in guild.roles.iter() {
                        if role_id.as_u64() == &old_id {
                            println!("Found role: {}", role.name);
                            role_wrap = Some(role);
                            break;
                        }
                    }
                    if let Some(role) = role_wrap {
                        println!("Renaming role...");
                        match role.edit(&ctx, |r| r.name(new_name)) {
                            Ok(_) => msg.react(&ctx.http, REACT_SUCCESS)?,
                            Err(err) => {
                                msg.react(&ctx.http, REACT_FAIL)?;
                                eprintln!("Error encountered: {}", err);
                            }
                        }
                    } else {
                        msg.react(&ctx.http, REACT_FAIL)?;
                        eprintln!("Failed to locate role")
                    }
                } else {
                    eprintln!("Failed to cache guild");
                    msg.react(&ctx.http, REACT_FAIL)?;
                }
            } else {
                msg.react(&ctx.http, REACT_FAIL)?;
            }
        } else {
            eprintln!("Failed to parse ID");
            msg.react(&ctx.http, REACT_FAIL)?;
        }
    } else {
        eprintln!("Message is too short");
        msg.react(&ctx.http, REACT_FAIL)?;
    }
    Ok(())
}

#[command]
fn rverify(ctx: &mut Context, msg: &Message) -> CommandResult {
    println!("DEBUG VERIFICATION REQUEST");
    if let Some(guild_id) = msg.guild_id {
        if let Some(guild) = guild_id.to_guild_cached(&ctx.cache) {
            msg.channel_id.send_message(&ctx.http, |response| {
                response.embed(|embed| {
                    let mut color = Color::BLUE;
                    let e = embed
                        .title("Role verification")
                        .author(|a| {
                            a.name(&CONFIG.name)
                                .icon_url(CurrentUser::face(&ctx.http.get_current_user().unwrap()))
                        })
                        .thumbnail(&CONFIG.thumbnail);
                    for (role, _) in CONFIG.public_roles.iter() {
                        if let Some(success_role) = guild.read().role_by_name(role) {
                            e.field(&success_role.name, "SUCCESS: Found role.", false);
                        // TODO Check role permissions
                        } else {
                            // TODO Fuzzy search for similar roles
                            e.field(role, "ERROR: Role not found!", false);
                            color = Color::DARK_RED;
                        }
                    }
                    e.color(color);
                    e
                });
                response
            })?;
        } else {
            eprintln!("Failed to cache guild");
            msg.react(&ctx.http, REACT_FAIL)?;
        }
    } else {
        msg.react(&ctx.http, REACT_FAIL)?;
    }
    Ok(())
}

#[command]
fn roles(ctx: &mut Context, msg: &Message) -> CommandResult {
    if !CONFIG.public_roles.is_empty() {
        println!("Printing role list...");
        msg.channel_id.send_message(&ctx.http, |response| {
            response.embed(|embed| {
                let e = embed
                    .title("Roles")
                    .author(|a| {
                        a.name(&CONFIG.name)
                            .icon_url(CurrentUser::face(&ctx.http.get_current_user().unwrap()))
                    })
                    .color(Color::FADED_PURPLE)
                    .thumbnail(&CONFIG.thumbnail);
                for (name, description) in CONFIG.public_roles.iter() {
                    e.field(name, description, true);
                }
                e
            });
            response
        })?;
    } else {
        msg.react(&ctx.http, REACT_FAIL)?;
    }
    Ok(())
}
