use crate::lexer::{Token, Position, TokenKind, Symbol, Keyword};

pub struct Compiler {
    /// Compiler state
    state: CompilerState,

    /// Buffer for the main function
    out_main: Vec<u8>,

    /// The number of args of main
    main_args_len: usize,

    /// Indexes of main args identifiers
    main_args_identifiers: Vec<String>,

    /// Buffer for other functions
    out_funcs: Vec<Vec<u8>>,

    /// Buffer for the bitmap
    out_bitmap: Vec<u8>,

    /// Number of bools written in the bitmap
    out_malleable_args_count: usize,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            state: CompilerState::Any,
            out_main: vec![],
            out_funcs: vec![],
            out_bitmap: vec![],
            main_args_identifiers: vec![],
            out_malleable_args_count: 0,
            main_args_len: 0,
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

            // We hit a malleable arg keyword
            (&CompilerState::ExpectingMainFuncMalleableOrIdentifier, TokenKind::Keyword(Keyword::Malleable)) => {
                // Increment bitmap count 
                self.out_malleable_args_count += 1;
                let desired_bitmap_len = (self.out_malleable_args_count - 1) / 8 + 1;

                // Add new bitmap to the buffer 
                if self.out_bitmap.len() < desired_bitmap_len {
                    self.out_bitmap.push(0x00);
                }
                
                let bitmap_len = self.out_bitmap.len();
                let bitmap_idx = self.out_malleable_args_count - 1;
                let bitmap = self.out_bitmap.get_mut(bitmap_len - 1).unwrap();
                *bitmap |= 1 << bitmap_idx;

                self.state = CompilerState::ExpectingMainFuncMalleableIdentifier;
            }

            // We didn't hit a malleable arg keywork but we hit an identifier
            (&CompilerState::ExpectingMainFuncMalleableOrIdentifier, TokenKind::Identifier(identifier)) => {
                self.main_args_len += 1;
                self.main_args_identifiers.push(identifier);
                self.state = CompilerState::ExpectingMainFuncMalleableOrIdentifier;
            }

            (&CompilerState::ExpectingMainFuncMalleableOrIdentifier, _) => {
                return Err(CompilerErr::ExpectedIdentifier(token.position.clone()));
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