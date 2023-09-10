use crate::compiler::Compiler;
use crate::lexer::tokenise;
use glob::glob;
use std::fs;

fn main() {
    let mut tokens: Vec<_> = vec![];

    for entry in glob("./**/*.ps").expect("Failed to read .ps files") {
        match entry {
            Ok(path) => {
                let contents =
                    fs::read_to_string(path).expect("Should have been able to read the file");
                tokens.extend(tokenise(&contents));
            }
            Err(e) => println!("{:?}", e),
        }
    }

    let mut compiler = Compiler::new();

    for t in tokens {
        match compiler.push_token(t) {
            Ok(()) => {}
            Err(err) => {
                println!("Compiler err: {:?}", err); // TODO: Pretty errors
            }
        }
    }

    println!(
        "Compiled successfuly! Output: \n\n{}",
        hex::encode(compiler.compile())
    );
}

mod compiler;
mod lexer;
