use std::{fs, path::PathBuf};

use ariadne::{Label, Report, ReportKind, Source, Span};
use chumsky::{chain::Chain, Parser as CParser, Stream};
use clap::{Args, Parser, Subcommand};
use crust::{Ast, Token};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Build(BuildArgs),
    Run(RunArgs),
}

#[derive(Args, Debug)]
struct BuildArgs {
    input: PathBuf,
    output: PathBuf,
}

#[derive(Args, Debug)]
#[command(version, about)]
struct RunArgs {
    input: PathBuf,
}

fn main() {
    match Cli::parse().commands {
        Commands::Build(args) => build(args),
        Commands::Run(args) => run(args),
    }
}

fn build(args: BuildArgs) {
    let source = match fs::read_to_string(&args.input) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Failed to read file: {e}");
            return;
        }
    };

    let source_len = source.chars().len();
    let filename = args
        .input
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
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

    if !parse_errors.is_empty() || !lexer_errors.is_empty() {
        std::process::exit(-1);
    }

    let serialized = match serde_json::to_string_pretty(&ast) {
        Ok(s) => s,
        Err(e) => {
            eprint!("Failed to serialize AST: {e}");
            return;
        }
    };

    fs::write(args.output, serialized).unwrap();
}

fn run(args: RunArgs) {
    let source = match fs::read_to_string(&args.input) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Failed to read file: {e}");
            return;
        }
    };

    let ast = match serde_json::from_str::<Ast>(&source) {
        Ok(ast) => ast,
        Err(e) => {
            eprint!("Error reading C IR: {e}");
            return;
        }
    };

    let exit_code = ast.run_main();
    println!("-- exited with code : {exit_code} --");
}
