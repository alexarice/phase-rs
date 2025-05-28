use std::path::PathBuf;

use phase_rs::parsing::tm;
use winnow::LocatingSlice;
use winnow::Parser;

/// Interpreter for "it's just a phase"
#[derive(clap::Parser)]
struct Args {
    /// File name to run
    #[arg(value_name = "FILE")]
    file: PathBuf,
}

fn main() {
    let args: Args = clap::Parser::parse();

    // let src_name = args.file.to_str().unwrap();
    let src = std::fs::read_to_string(&args.file).unwrap();

    match tm.parse(LocatingSlice::new(&src)) {
        Ok(t) => println!("Type: {t:?}"),
        Err(e) => eprintln!("{e}"),
    }
}
