use glob::glob;
use std::fs;
use crate::{lexer::tokenise, ast::{Function, Program}};

fn main() {
    let mut tokens: Vec<Function> = vec![];

    for entry in glob("./**/*.ps").expect("Failed to read .ps files") {
        match entry {
            Ok(path) => {
                let contents = fs::read_to_string(path)
                    .expect("Should have been able to read the file");
                let r: Vec<_> = tokenise(&contents).collect();
                tokens.push(r.into());
            },
            Err(e) => println!("{:?}", e),
        }
    }
    let program: Program = tokens.into();
    println!("{:#?}", program);
}

mod lexer;
mod compiler;
mod ast;