use anyhow::Result;
use tokio;


#[tokio::main]
pub async fn main() -> Result<()> {
 
    let mut args = std::env::args();
    if args.len() != 3 {
        eprintln!("Usage: rasta <rocket server> <token>");
        return Ok(())
    }

    args.next();

    let hostname = args.next().unwrap();
    let creds = rasta::Credentials::Token(args.next().unwrap());

    let mut cli = rasta::Rasta::connect(&hostname).await?;
    let _tokens = cli.login(creds).await?;

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