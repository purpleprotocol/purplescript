use glob::glob;
use std::fs;
use crate::lexer::tokenise;

fn main() {
    let mut tokens: Vec<Vec<_>> = vec![];

    for entry in glob("./**/*.ps").expect("Failed to read .ps files") {
        match entry {
            Ok(path) => {
                let contents = fs::read_to_string(path)
                    .expect("Should have been able to read the file");
                tokens.push(tokenise(&contents).collect());
            },
            Err(e) => println!("{:?}", e),
        }
    }
    println!("{:#?}", tokens);
}

mod lexer;
mod ast;