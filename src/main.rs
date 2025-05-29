use std::{io, io::Read, path::PathBuf};

use float_pretty_print::PrettyPrintFloat;
use phase_rs::parsing::command;
use winnow::{LocatingSlice, Parser, ascii::multispace0, combinator::terminated};

/// Interpreter for "it's just a phase"
#[derive(clap::Parser)]
struct Args {
    /// File name to run
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,
}

fn parse_and_check(src: &str) -> Result<(), String> {
    let parsed = terminated(command, multispace0)
        .parse(LocatingSlice::new(src))
        .map_err(|e| format!("{e}"))?;
    println!("Parsed: {parsed:?}");
    let (_env, checked) = parsed.check().map_err(|e| format!("{e:?}"))?;
    println!("Checked: {checked:?}");
    let unitary = checked.to_unitary();
    for x in unitary.row_iter() {
        println!(
            "[ {} ]",
            x.iter()
                .map(|x| {
                    match (x.re.abs() > 0.000001, x.im.abs() > 0.000001) {
                        (false, false) => "0.0".to_owned(),
                        (true, false) => format!("{}", PrettyPrintFloat(x.re)),
                        (false, true) => format!("{}i", PrettyPrintFloat(x.im)),
                        (true, true) => {
                            format!("{} + {}i", PrettyPrintFloat(x.re), PrettyPrintFloat(x.im))
                        }
                    }
                })
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
    Ok(())
}

fn main() {
    let args: Args = clap::Parser::parse();

    let src = if let Some(path) = &args.file {
        std::fs::read_to_string(path).unwrap()
    } else {
        let mut s = String::new();
        io::stdin().read_to_string(&mut s).unwrap();
        s
    };

    if let Err(e) = parse_and_check(&src) {
        eprintln!("{}", e)
    }
}
