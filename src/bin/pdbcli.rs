use anyhow::{anyhow, Error, Result};
use pdb::parse::parse;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use serde_lexpr::to_string;
use structopt::StructOpt;
use tokio::io::AsyncBufRead;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::io::BufWriter;
use tokio::net::TcpStream;

const PROMPT: &str = ">> ";

#[derive(Debug, StructOpt)]
struct Config {
    #[structopt(short = "d", env = "DATABASE_URL")]
    database_url: String,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "pdbcli", about = "A simple client for pdb.")]
struct Args {
    #[structopt(flatten)]
    config: Config,
}

#[paw::main]
#[tokio::main]
async fn main(args: Args) -> Result<()> {
    println!("Welcome to pdbcli!");

    let stream = TcpStream::connect(&args.config.database_url).await?;
    let stream = BufWriter::new(BufReader::new(stream));

    println!("Connected to {}!", &args.config.database_url);

    start(args.config, stream).await?;

    Ok(())
}

async fn start(_config: Config, mut stream: impl AsyncWrite + AsyncBufRead + Unpin) -> Result<()> {
    let mut rl = Editor::<()>::new();

    loop {
        let readline = rl.readline(PROMPT);
        match readline {
            Ok(line) => {
                match parse(&line) {
                    Ok(stm) => {
                        let mut sexp = to_string(&stm)?;
                        sexp.push('\n');
                        stream.write_all(sexp.as_bytes()).await?;
                        stream.flush().await?;

                        let mut buffer = String::new();

                        let _ = stream.read_line(&mut buffer).await?;

                        println!("{}", buffer);
                    }
                    Err(e) => {
                        println!("No parse: {}\n", e);
                    }
                };
                rl.add_history_entry(line);
            }
            Err(ReadlineError::Interrupted) => {
                break Err(anyhow!("unimplemented"));
            }
            Err(ReadlineError::Eof) => {
                return Ok(());
            }
            Err(err) => break Err(Error::new(err)),
        }
    }
}
