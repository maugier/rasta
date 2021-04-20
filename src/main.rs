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

    loop {
        let msg = cli.recv().await?;
        eprintln!("Got message: {:?}", msg)
    }

}