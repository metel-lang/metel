//! Integration tests: each .gust test program must run through the parser without errors. These tests are meant to cover the full range of language features, and are not expected to be minimal. They are primarily intended to catch regressions in the parser as new features are added.

use gust::parser;

fn run(filename: &str) {
    let path = format!("tests/parsing/sources/{}", filename);
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("could not read {}: {}", path, e));
    parser::parse(&source, &path)
        .unwrap_or_else(|e| panic!("{}", e));
}

#[test]
fn test_01_literals_and_variables() { run("01_literals_and_variables.gust"); }

#[test]
fn test_02_control_flow() { run("02_control_flow.gust"); }

#[test]
fn test_03_functions_and_closures() { run("03_functions_and_closures.gust"); }

#[test]
fn test_04_structs_and_impl() { run("04_structs_and_impl.gust"); }

#[test]
fn test_05_enums_and_match() { run("05_enums_and_match.gust"); }

#[test]
fn test_06_traits() { run("06_traits.gust"); }

#[test]
fn test_07_arrays_and_tuples() { run("07_arrays_and_tuples.gust"); }

#[test]
fn test_08_error_handling() { run("08_error_handling.gust"); }

#[test]
fn test_09_casting_and_generics() { run("09_casting_and_generics.gust"); }

#[test]
fn test_10_comprehensive() { run("10_comprehensive.gust"); }

#[test]
fn test_11_block_expr_stmts() { run("11_block_expr_stmts.gust"); }
