use anyhow::Result;
use tokio;
use rasta::schema::Room;

#[tokio::main]
pub async fn main() -> Result<()> {
 
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        panic!("Incorrect arguments")
    }

    let hostname = &args[1];
    let creds = rasta::Credentials::Token(args[2].to_string());

    let mut cli = rasta::Rasta::connect(hostname).await?;
    let _tokens = cli.login(creds).await?;

    for room in cli.rooms().await? {
        if let Room::Chat { name, .. } = room {
            eprintln!("*** Room #{}", name);
        }
    }

    loop {
        let msg = cli.recv().await?;
        eprintln!("Got message: {:?}", msg)
    }

}