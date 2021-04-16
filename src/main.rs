use anyhow::Result;
use tokio;

#[tokio::main]
pub async fn main() -> Result<()> {
 
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 4 {
        panic!("Incorrect arguments")
    }

    let hostname = &args[1];
    let creds = rasta::Credentials::Clear { user: args[2].to_string() , password: args[3].to_string() };

    let mut cli = rasta::Rasta::connect(hostname).await?;
    let _tokens = cli.login(creds).await?;

    loop {
        let msg = cli.recv().await?;
        eprintln!("Got message: {:?}", msg)
    }

}