use glob::glob;
use std::fs;
use crate::lexer::tokenise;
use crate::compiler::Compiler;

fn main() {
    let mut tokens: Vec<_> = vec![];
    let mut out_script: Vec<u8> = vec![];

    for entry in glob("./**/*.ps").expect("Failed to read .ps files") {
        match entry {
            Ok(path) => {
                let contents = fs::read_to_string(path)
                    .expect("Should have been able to read the file");
                tokens.extend(tokenise(&contents));
            },
            Err(e) => println!("{:?}", e),
        }
    }

    let mut compiler = Compiler::new();

    for t in tokens {
        match compiler.push_token(t) {
            Ok(()) => { },
            Err(err) => {
                println!("Compiler err: {:?}", err); // TODO: Pretty errors
            } 
        }
    }

    println!("Compiled successfuly! Output: \n\n{}", hex::encode(compiler.compile()));
}

mod lexer;
mod compiler;