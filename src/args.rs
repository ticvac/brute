use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "DSVA Node")]
#[command(about = "Computing node for DSVA", long_about = None)]
pub struct Args {
    /// Port to listen on
    #[arg(short, long)]
    pub port: u16,

    /// List of friends to connect to (format: port or ip:port)
    #[arg(short, long, value_delimiter = ',')]
    pub friends: Vec<String>,

    /// Faker mode: report only one tenth of actual power
    #[arg(long, default_value_t = false)]
    pub faker: bool,
}
