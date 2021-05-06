use anyhow::Result;
use tokio;


#[tokio::main]
pub async fn main() -> Result<()> {
 
    let mut args = std::env::args();
    if args.len() != 3 {
        eprintln!("Usage: rasta <rocket server> <credential>");
        return Ok(())
    }

    args.next();

    let hostname = args.next().unwrap();
    let creds = rasta::Credentials::from(args.next().unwrap());

    debug!("Using credentials {:?}", creds);

    let mut rest = rasta::rest::Client::new(&hostname);

    let creds2 = rest.login(&creds).await?;

    let mut cli = rasta::Rasta::connect(&hostname).await?;
    let _tokens = cli.login(creds2).await?;

    let chans = cli.rooms().await?;

    for chan in chans {
        eprintln!("Room {:?}", chan);

        match rest.channel_members(&chan).await {
            Ok(members) => eprintln!("Members: {:?}", members),
            Err(e) => eprintln!("Could not get members: {}", e),
        }

    }

    cli.subscribe_my_messages().await?;
    cli.subscribe("stream-notify-logged".into(), vec!["user-status".into()]).await?;
    cli.subscribe("stream-notify-user".into(), vec!["syn/rooms-changed".into()] ).await?;


    let session = rasta::session::Session::from(&mut cli).await?;
    eprintln!("\n\nSession info:\n{:?}", session);

    loop {
        let msg = cli.recv().await?;
        eprintln!("Got message: {}", msg.pretty());
    }

}