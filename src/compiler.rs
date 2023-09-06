use crate::lexer::{Token, Position, TokenKind, Symbol, Keyword};

pub struct Compiler {
    /// Compiler state
    state: CompilerState,

    /// Buffer for the main function
    out_main: Vec<u8>,

    /// Buffer for other functions
    out_funcs: Vec<Vec<u8>>,

    /// Buffer for the bitmap
    out_bitmap: Vec<u8>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            state: CompilerState::Any,
            out_main: vec![],
            out_funcs: vec![],
            out_bitmap: vec![],
        }
    }

    pub fn push_token(&mut self, token: Token) -> Result<(), CompilerErr> {
        match (&self.state, token.kind) {
            (&CompilerState::Any, TokenKind::Keyword(Keyword::Function)) => {
                self.state = CompilerState::ExpectingFuncIdentifier;
            }

            // We expected a function identifier
            (&CompilerState::Any, _) => {
                return Err(CompilerErr::ExpectedFunctionDefinition(token.position.clone()));
            }

            // Main function identifier
            (&CompilerState::ExpectingFuncIdentifier, TokenKind::Identifier(identifier)) if identifier.as_str() == "main" => {
                self.state = CompilerState::ExpectingMainFuncLeftParanthesis;
            }

            // Other function identifier
            (&CompilerState::ExpectingFuncIdentifier, TokenKind::Identifier(identifier)) if identifier.as_str() != "main" => {
                self.state = CompilerState::ExpectingFuncLeftParanthesis; 
            }

            (&CompilerState::ExpectingFuncIdentifier, _) => {
                return Err(CompilerErr::ExpectedIdentifier(token.position.clone()));
            }

            (&CompilerState::ExpectingMainFuncLeftParanthesis, TokenKind::Symbol(Symbol::ParenthesisLeft)) => {
                self.state = CompilerState::ExpectingMainFuncMalleableOrIdentifier;
            }

            (&CompilerState::ExpectingFuncLeftParanthesis, TokenKind::Symbol(Symbol::ParenthesisLeft)) => {
                self.state = CompilerState::ExpectingFuncArgIdentifier;
            }

            (&CompilerState::ExpectingFuncLeftParanthesis, _) => {
                return Err(CompilerErr::ExpectedParanthesisLeft(token.position.clone()));
            }

            _ => unimplemented!()
        }


        Ok(())
    }

    pub fn compile(&self) -> Vec<u8> {
        unimplemented!();
    }
}

#[derive(Debug, Clone)]
pub enum CompilerErr {
    ExpectedFunctionDefinition(Position),
    ExpectedIdentifier(Position),
    ExpectedParanthesisLeft(Position),
}

enum CompilerState {
    // General states
    //
    
    /// Any func definition
    Any,

    /// Any func definition except the main function
    AnyExceptMain,

    // Func definition states
    //
    
    /// We hit a `function` definition, now we want the identifier.
    ExpectingFuncIdentifier,

    /// We hit the main function identifier, now we want the left paranthesis for the arguments.
    ExpectingMainFuncLeftParanthesis,

    /// We hit the main function paranthesis for arguments, now we want the actual 
    /// arguments which can be malleable or not.
    ExpectingMainFuncMalleableOrIdentifier,

    /// We hit a malleable arg definition, now we want the identifier.
    ExpectingMainFuncMalleableIdentifier,

    /// We have hit the main function identifier, now we either expect a colon,
    /// in order to specify the argument type, a comma, or the right paranthesis
    /// to end the main function definitions.
    ExpectingMainFuncColonCommaOrRightParanthesis,

    /// We finished the arguments definition, now we want the start brace
    /// of the main function body.
    ExpectingMainFuncBrace,

    /// We hit the start brace of the function body, now we want the actual body.
    ExpectingMainFuncBody,

    /// We hit the function identifier, now we want the left paranthesis for the arguments.
    ExpectingFuncLeftParanthesis,

    /// We hit the function paranthesis for arguments, now we want the actual 
    /// arguments.
    ExpectingFuncArgIdentifier,

    /// We have hit the function identifier, now we either expect a colon,
    /// in order to specify the argument type, a comma, or the right paranthesis
    /// to end the function definitions.
    ExpectingFuncColonCommaOrRightParanthesis,

    /// We finished the arguments definition, now we want the start brace
    /// of the function body.
    ExpectingFuncBrace,

    /// We hit the start brace of the function body, now we want the actual body.
    ExpectingFuncBody,

    // Func bodies
    //
}