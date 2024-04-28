use std::{fs, path::PathBuf};

use ariadne::{Label, Report, ReportKind, Source, Span};
use chumsky::{chain::Chain, Parser as CParser, Stream};
use clap::Parser;
use crust::{Ast, Token};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    file: PathBuf,
}

fn main() {
    let args = Args::parse();
    let source = match fs::read_to_string(&args.file) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Failed to read file: {e}");
            return;
        }
    };

    let source_len = source.chars().len();
    let filename = args.file.file_name().unwrap().to_string_lossy().to_string();
    let (Some(tokens), lexer_errors) = Token::lexer().parse_recovery(source.as_str()) else {
        eprintln!("Failed to generate tokens...");
        return;
    };

    for error in lexer_errors.iter() {
        Report::build(ReportKind::Error, &filename, error.span().start())
            .with_message("Lexer Error")
            .with_label(Label::new((&filename, error.span())).with_message(&error))
            .finish()
            .eprint((&filename, Source::from(source.as_str())))
            .unwrap();
    }

    let (ast, parse_errors) = Ast::parser().parse_recovery(Stream::from_iter(
        source_len..source_len + 1,
        tokens.into_iter(),
    ));

    for error in parse_errors.iter() {
        Report::build(ReportKind::Error, &filename, error.span().start())
            .with_message("Parser Error")
            .with_label(Label::new((&filename, error.span())).with_message(&error))
            .finish()
            .eprint((&filename, Source::from(source.as_str())))
            .unwrap();
    }

    println!("{ast:?}")
}
