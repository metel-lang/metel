use std::process;

use clap::Parser;

mod ast;
mod error;
mod evaluator;
mod module_loader;
mod name_resolver;
mod parser;
mod path_normalizer;
mod typed_ast;
mod typechecker;
mod typeinference;
mod types;

use error::MoonlaneError;

#[derive(Parser)]
#[command(name = "moonlane")]
#[command(version = "0.1.0")]
#[command(about = "Moonlane interpreter")]
#[command(long_about = "A tree-walk interpreter for the Moonlane programming language")]
struct Args {
    /// Path to the \.mln file to execute
    #[arg(value_name = "FILE")]
    file: String,

    /// Print the AST and exit without executing
    #[arg(long)]
    debug_ast: bool,
}

fn main() {
    let args = Args::parse();

    if let Err(e) = run(&args.file, args.debug_ast) {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn run(filename: &str, debug_ast: bool) -> Result<(), MoonlaneError> {
    // 1. Load modules → untyped AST
    let ast = module_loader::load_program(filename)?;

    if debug_ast {
        println!("{:#?}", ast);
        return Ok(());
    }

    // 2. Type check → typed AST
    let typed_ast = typechecker::check(ast)?;

    

    // 3. Evaluate
    evaluator::evaluate(typed_ast)?;

    Ok(())
}
