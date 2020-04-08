use serenity::client::Client;
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::{Context, EventHandler};

#[command]
fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!")?;

    Ok(())
}

// TODO Handle unwrap()
#[command]
fn role(ctx: &mut Context, msg: &Message) -> CommandResult {
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
                member.remove_role(&ctx.http, role.id)?;
            } else {
                println!("Adding role {} to user...", &role.name);
                member.add_role(&ctx.http, role.id)?;
            }
        } // TODO Handle role not found
    } // TODO Handle failure to get the guild info

    Ok(())
}

#[group]
#[commands(ping, role)]
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
