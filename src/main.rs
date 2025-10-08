use std::{io, io::Read, path::PathBuf};

use float_pretty_print::PrettyPrintFloat;
use miette::{Result, miette};
use phase_rs::{
    command::Command,
    normal_syntax::TermN,
    text::{HasParser, ToDoc},
};
use winnow::{LocatingSlice, Parser};

/// Interpreter for "it's just a phase"
#[derive(clap::Parser)]
struct Args {
    /// File name to run
    #[arg(long, value_name = "FILE")]
    file: Option<PathBuf>,
}

fn parse_and_check(src: &str) -> Result<()> {
    let parsed = Command::parser
        .parse(LocatingSlice::new(src))
        .map_err(|e| miette!("{e}"))?;
    let (_env, checked) = parsed.check()?;
    println!("Input term:\n{}\n", checked.to_raw().to_doc().pretty(60));
    let mut evalled: TermN = checked.eval();
    evalled.squash();
    let quoted = evalled.quote();
    let raw = quoted.to_raw();
    println!("Evaluated:\n{}\n", raw.to_doc().pretty(60));
    let circuit = quoted.eval_circ();
    let circuit_quoted = circuit.quote();
    let circuit_raw = circuit_quoted.to_raw();
    println!("Circuit:\n{}\n", circuit_raw.to_doc().pretty(60));
    let unitary = evalled.to_unitary();
    println!("Unitary:");
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

fn main() -> Result<()> {
    let args: Args = clap::Parser::parse();

    let src = if let Some(path) = &args.file {
        std::fs::read_to_string(path).unwrap()
    } else {
        let mut s = String::new();
        io::stdin().read_to_string(&mut s).unwrap();
        s
    };

    parse_and_check(&src).map_err(|e| e.with_source_code(src))?;

    Ok(())
}
