use serenity::client::Client;
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};
use serenity::model::{channel::Message, channel::ReactionType, gateway::Ready};
use serenity::prelude::{Context, EventHandler};
use serenity::utils::MessageBuilder;

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
    let mut client = Client::new(&std::env::var("DISCORD_TOKEN").expect("token"), Handler)
        .expect("Error creating client");
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
    // These should probably be const, but serenity uses String for some reason
    let react_success = ReactionType::Unicode(String::from("✅"));
    let react_fail = ReactionType::Unicode(String::from("🟥"));

    let role_name = msg.content.split(" ").collect::<Vec<&str>>()[1];
    let mut member = msg
        .guild_id
        .unwrap()
        .member(&ctx.http, msg.author.id)
        .unwrap();

    if let Some(arc) = msg.guild_id.unwrap().to_guild_cached(&ctx.cache) {
        if let Some(role) = arc.read().role_by_name(role_name) {
            if msg.member.as_ref().unwrap().roles.contains(&role.id) {
                println!("Removing role {} from user...", &role.name);
                let reaction = match member.remove_role(&ctx.http, role.id) {
                    Ok(_) => react_success,
                    Err(_) => react_fail,
                };
                msg.react(&ctx.http, reaction)?;
            } else {
                println!("Adding role {} to user...", &role.name);
                let reaction = match member.add_role(&ctx.http, role.id) {
                    Ok(_) => react_success,
                    Err(_) => react_fail,
                };
                msg.react(&ctx.http, reaction)?;
            }
        } else {
            msg.react(&ctx.http, react_fail)?;
        }
    } else {
        msg.react(&ctx.http, react_fail)?;
    }
    Ok(())
}
