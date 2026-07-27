use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

// modules
mod core;
mod lexer;
mod parser;

// imports
use core::{ASTNode, ParserState, Text};
use lexer::scan;
use parser::parse;

fn main() -> std::io::Result<()> {
    let file = File::open("source.txt")?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;

    if contents.is_empty() {
        return Ok(());
    }

    let mut text = Text {
        source: contents,
        current: 0,
        length: 0,
    };

    text.length = text.source.len();

    let mut tokens = Vec::new();

    scan(&mut text, &mut tokens);

    println!("Tokens: {:#?}", tokens);

    let mut parserstate = ParserState {
        tokens,
        position: 0,
    };

    let mut ast = Vec::<ASTNode>::new();

    parse(&mut parserstate, &mut ast);

    println!("AST: {:#?}", ast);

    Ok(())
}