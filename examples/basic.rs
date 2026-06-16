use twitch_badge_parser_rs::TwitchBadgeParser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parser = TwitchBadgeParser::new("token", "client-id").await?;

    let badge1 = parser.get("zevent25");
    parser.add_channel("80585512").await?;
    let sub_badge = parser.get("subscriber");
    parser.remove_channel("80585512");
    let sub_badge2 = parser.get("subscriber");

    println!("badge: {:#?}", badge1);
    println!("sub_badge: {:#?}", sub_badge);
    println!("sub_badge2: {:#?}", sub_badge2);

    Ok(())
}
