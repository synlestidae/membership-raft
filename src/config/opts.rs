use clap::Clap;

/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields
#[derive(Clap, Debug)]
#[clap(version = "0.1", author = "Mate Antunovic")]
pub struct Opts {
    #[clap(short = "c", long = "config")]
    pub config: Option<String>,

    #[clap(short = "n", long = "nodeid")]
    pub node_id: Option<u64>,
}
