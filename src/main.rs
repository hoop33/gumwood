use gumwood::{self, Options};
use std::process;
use structopt::StructOpt;

fn main() {
    let args = Options::from_args();

    if let Err(e) = gumwood::run(args) {
        eprintln!("error: {}", e);
        process::exit(1);
    }
}
