use anyhow::Result;

#[tokio::main]
pub async fn main() -> Result<()> {
 
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 4 {
        panic!("Incorrect arguments")
    }

    let mut cli = rasta::Rasta::new(&args[1])?;
    cli.login(args[2].to_owned(), args[3].to_owned()).await?;
    
    let chans = cli.channels().await?;

    eprintln!("Your channels are:");
    for c in &chans {
        eprintln!("#{} ({:?})", c.name, c);
    }

    Ok(())
}