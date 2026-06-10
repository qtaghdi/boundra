mod cli;

use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    std::process::exit(cli::run(&args));
}
