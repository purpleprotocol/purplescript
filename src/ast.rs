use crate::lexer::Token;

#[derive(Clone, Debug)]
pub struct Function {
    tokens: Vec<Token>,
}

impl From<Vec<Token>> for Function {
    fn from(other: Vec<Token>) -> Self {
        Self {
            tokens: other
        }
    }
}

#[derive(Clone, Debug)]
pub struct Program {
    functions: Vec<Function>,
}

impl From<Vec<Function>> for Program {
    fn from(other: Vec<Function>) -> Self {
        Self {
            functions: other,
        }
    }
}