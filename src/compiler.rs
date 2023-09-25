use crate::lexer::{Keyword, Position, Symbol, Token, TokenKind};

type IdentifierAndArg = (String, ValueType);
type FuncIdx = usize; // 0 for main
type Frame = (FuncIdx, Vec<IdentifierAndArg>);

pub struct Compiler {
    /// Compiler state
    state: CompilerState,

    /// Buffer for the main function
    out_main: Vec<u8>,

    /// Use a frame stack to keep track of definitions and to
    /// translate them to frames.
    frame_stack: Vec<Frame>,

    /// Buffer for other functions
    out_funcs: Vec<Vec<u8>>,

    /// Buffer for the bitmap
    out_bitmap: Vec<u8>,

    /// Number of bools written in the bitmap
    out_malleable_args_count: usize,

    /// If we found the main function or not
    found_main: bool,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            state: CompilerState::Any,
            out_main: vec![],
            out_funcs: vec![],
            out_bitmap: vec![],
            frame_stack: vec![(0, vec![])],
            out_malleable_args_count: 0,
            found_main: false,
        }
    }

    pub fn push_token(&mut self, token: Token) -> Result<(), CompilerErr> {
        match (&self.state, token.kind) {
            (&CompilerState::Any, TokenKind::Keyword(Keyword::Function)) => {
                self.state = CompilerState::ExpectingFuncIdentifier;
            }

            // We expected a function identifier
            (&CompilerState::Any, _) => {
                return Err(CompilerErr::ExpectedFunctionDefinition(
                    token.position.clone(),
                ));
            }

            // Main function identifier
            (&CompilerState::ExpectingFuncIdentifier, TokenKind::Identifier(identifier))
                if identifier.as_str() == "main" =>
            {
                if self.found_main {
                    return Err(CompilerErr::DuplicateMainDeclaration(
                        token.position.clone(),
                    ));
                }

                self.found_main = true;
                self.state = CompilerState::ExpectingMainFuncLeftParanthesis;
            }

            // Other function identifier
            (&CompilerState::ExpectingFuncIdentifier, TokenKind::Identifier(identifier))
                if identifier.as_str() != "main" =>
            {
                self.state = CompilerState::ExpectingFuncLeftParanthesis;
            }

            (&CompilerState::ExpectingFuncIdentifier, _) => {
                return Err(CompilerErr::ExpectedIdentifier(token.position.clone()));
            }

            (
                &CompilerState::ExpectingMainFuncLeftParanthesis,
                TokenKind::Symbol(Symbol::ParenthesisLeft),
            ) => {
                self.state = CompilerState::ExpectingMainFuncMalleableOrIdentifier;
            }

            (
                &CompilerState::ExpectingFuncLeftParanthesis,
                TokenKind::Symbol(Symbol::ParenthesisLeft),
            ) => {
                self.state = CompilerState::ExpectingFuncArgIdentifier;
            }

            (&CompilerState::ExpectingFuncLeftParanthesis, _) => {
                return Err(CompilerErr::ExpectedLeftParanthesis(token.position.clone()));
            }

            // We hit a malleable arg keyword
            (
                &CompilerState::ExpectingMainFuncMalleableOrIdentifier,
                TokenKind::Keyword(Keyword::Malleable),
            ) => {
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
            (
                &CompilerState::ExpectingMainFuncMalleableOrIdentifier,
                TokenKind::Identifier(identifier),
            ) => {
                self.frame_stack
                    .last_mut()
                    .unwrap()
                    .1
                    .push((identifier, ValueType::Any));
                self.state = CompilerState::ExpectingMainFuncColonCommaOrRightParanthesis;
            }

            // We hit the right paranthesis after a comma. This is valid.
            (
                &CompilerState::ExpectingMainFuncMalleableOrIdentifier,
                TokenKind::Symbol(Symbol::ParenthesisRight),
            ) => {
                self.state = CompilerState::ExpectingMainFuncBrace;
            }

            (&CompilerState::ExpectingMainFuncMalleableOrIdentifier, _) => {
                return Err(CompilerErr::ExpectedIdentifier(token.position.clone()));
            }

            (
                &CompilerState::ExpectingMainFuncColonCommaOrRightParanthesis,
                TokenKind::Symbol(Symbol::Colon),
            ) => {
                self.state = CompilerState::ExpectingMainFuncArgType;
            }

            (
                &CompilerState::ExpectingMainFuncColonCommaOrRightParanthesis
                | &CompilerState::ExpectingMainFuncCommaOrRightParanthesis,
                TokenKind::Symbol(Symbol::Comma),
            ) => {
                self.state = CompilerState::ExpectingMainFuncMalleableOrIdentifier;
            }

            (
                &CompilerState::ExpectingMainFuncColonCommaOrRightParanthesis
                | &CompilerState::ExpectingMainFuncCommaOrRightParanthesis,
                TokenKind::Symbol(Symbol::ParenthesisRight),
            ) => {
                self.state = CompilerState::ExpectingFuncBrace;
            }

            (&CompilerState::ExpectingMainFuncColonCommaOrRightParanthesis, _) => {
                return Err(CompilerErr::ExpectedColonCommaOrRightParanthesis(
                    token.position.clone(),
                ));
            }

            (&CompilerState::ExpectingMainFuncCommaOrRightParanthesis, _) => {
                return Err(CompilerErr::ExpectedCommaOrRightParanthesis(
                    token.position.clone(),
                ));
            }

            (&CompilerState::ExpectingMainFuncBrace, TokenKind::Symbol(Symbol::BraceLeft)) => {
                self.state = CompilerState::ExpectingMainFuncBody;
            }

            (&CompilerState::ExpectingMainFuncBrace, _) => {
                return Err(CompilerErr::ExpectedLeftBrace(token.position.clone()));
            }

            // Main func argument types
            (&CompilerState::ExpectingMainFuncArgType, TokenKind::Keyword(Keyword::Address)) => {
                let (_, ref mut arg_type) =
                    self.frame_stack.last_mut().unwrap().1.last_mut().unwrap();
                *arg_type = ValueType::Address;
                self.state = CompilerState::ExpectingMainFuncCommaOrRightParanthesis;
            }

            (&CompilerState::ExpectingMainFuncArgType, TokenKind::Keyword(Keyword::Asset)) => {
                let (_, ref mut arg_type) =
                    self.frame_stack.last_mut().unwrap().1.last_mut().unwrap();
                *arg_type = ValueType::Asset;
                self.state = CompilerState::ExpectingMainFuncCommaOrRightParanthesis;
            }

            (&CompilerState::ExpectingMainFuncArgType, TokenKind::Keyword(Keyword::U8)) => {
                let (_, ref mut arg_type) =
                    self.frame_stack.last_mut().unwrap().1.last_mut().unwrap();
                *arg_type = ValueType::U8;
                self.state = CompilerState::ExpectingMainFuncCommaOrRightParanthesis;
            }

            (&CompilerState::ExpectingMainFuncArgType, TokenKind::Keyword(Keyword::U16)) => {
                let (_, ref mut arg_type) =
                    self.frame_stack.last_mut().unwrap().1.last_mut().unwrap();
                *arg_type = ValueType::U16;
                self.state = CompilerState::ExpectingMainFuncCommaOrRightParanthesis;
            }

            (&CompilerState::ExpectingMainFuncArgType, TokenKind::Keyword(Keyword::U32)) => {
                let (_, ref mut arg_type) =
                    self.frame_stack.last_mut().unwrap().1.last_mut().unwrap();
                *arg_type = ValueType::U32;
                self.state = CompilerState::ExpectingMainFuncCommaOrRightParanthesis;
            }

            (&CompilerState::ExpectingMainFuncArgType, TokenKind::Keyword(Keyword::U64)) => {
                let (_, ref mut arg_type) =
                    self.frame_stack.last_mut().unwrap().1.last_mut().unwrap();
                *arg_type = ValueType::U64;
                self.state = CompilerState::ExpectingMainFuncCommaOrRightParanthesis;
            }

            (&CompilerState::ExpectingMainFuncArgType, TokenKind::Keyword(Keyword::U128)) => {
                let (_, ref mut arg_type) =
                    self.frame_stack.last_mut().unwrap().1.last_mut().unwrap();
                *arg_type = ValueType::U128;
                self.state = CompilerState::ExpectingMainFuncCommaOrRightParanthesis;
            }

            (&CompilerState::ExpectingMainFuncArgType, TokenKind::Keyword(Keyword::UBIG)) => {
                let (_, ref mut arg_type) =
                    self.frame_stack.last_mut().unwrap().1.last_mut().unwrap();
                *arg_type = ValueType::UBIG;
                self.state = CompilerState::ExpectingMainFuncCommaOrRightParanthesis;
            }

            (&CompilerState::ExpectingMainFuncArgType, TokenKind::Keyword(Keyword::I8)) => {
                let (_, ref mut arg_type) =
                    self.frame_stack.last_mut().unwrap().1.last_mut().unwrap();
                *arg_type = ValueType::I8;
                self.state = CompilerState::ExpectingMainFuncCommaOrRightParanthesis;
            }

            (&CompilerState::ExpectingMainFuncArgType, TokenKind::Keyword(Keyword::I16)) => {
                let (_, ref mut arg_type) =
                    self.frame_stack.last_mut().unwrap().1.last_mut().unwrap();
                *arg_type = ValueType::I16;
                self.state = CompilerState::ExpectingMainFuncCommaOrRightParanthesis;
            }

            (&CompilerState::ExpectingMainFuncArgType, TokenKind::Keyword(Keyword::I32)) => {
                let (_, ref mut arg_type) =
                    self.frame_stack.last_mut().unwrap().1.last_mut().unwrap();
                *arg_type = ValueType::I32;
                self.state = CompilerState::ExpectingMainFuncCommaOrRightParanthesis;
            }

            (&CompilerState::ExpectingMainFuncArgType, TokenKind::Keyword(Keyword::I64)) => {
                let (_, ref mut arg_type) =
                    self.frame_stack.last_mut().unwrap().1.last_mut().unwrap();
                *arg_type = ValueType::I64;
                self.state = CompilerState::ExpectingMainFuncCommaOrRightParanthesis;
            }

            (&CompilerState::ExpectingMainFuncArgType, TokenKind::Keyword(Keyword::I128)) => {
                let (_, ref mut arg_type) =
                    self.frame_stack.last_mut().unwrap().1.last_mut().unwrap();
                *arg_type = ValueType::I128;
                self.state = CompilerState::ExpectingMainFuncCommaOrRightParanthesis;
            }

            (&CompilerState::ExpectingMainFuncArgType, TokenKind::Keyword(Keyword::IBIG)) => {
                let (_, ref mut arg_type) =
                    self.frame_stack.last_mut().unwrap().1.last_mut().unwrap();
                *arg_type = ValueType::IBIG;
                self.state = CompilerState::ExpectingMainFuncCommaOrRightParanthesis;
            }

            (&CompilerState::ExpectingMainFuncArgType, TokenKind::Keyword(Keyword::F32)) => {
                let (_, ref mut arg_type) =
                    self.frame_stack.last_mut().unwrap().1.last_mut().unwrap();
                *arg_type = ValueType::F32;
                self.state = CompilerState::ExpectingMainFuncCommaOrRightParanthesis;
            }

            (&CompilerState::ExpectingMainFuncArgType, TokenKind::Keyword(Keyword::F64)) => {
                let (_, ref mut arg_type) =
                    self.frame_stack.last_mut().unwrap().1.last_mut().unwrap();
                *arg_type = ValueType::F64;
                self.state = CompilerState::ExpectingMainFuncCommaOrRightParanthesis;
            }

            (&CompilerState::ExpectingMainFuncArgType, TokenKind::Keyword(Keyword::Decimal)) => {
                let (_, ref mut arg_type) =
                    self.frame_stack.last_mut().unwrap().1.last_mut().unwrap();
                *arg_type = ValueType::Decimal;
                self.state = CompilerState::ExpectingMainFuncCommaOrRightParanthesis;
            }

            _ => unimplemented!(),
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
    ExpectedLeftParanthesis(Position),
    ExpectedColonCommaOrRightParanthesis(Position),
    ExpectedCommaOrRightParanthesis(Position),
    ExpectedLeftBrace(Position),
    DuplicateMainDeclaration(Position),
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

    /// We have just parsed an argument type, so we allow either a comma or
    /// a right paranthesis.
    ExpectingMainFuncCommaOrRightParanthesis,

    /// We have hit a colon for a main function argument, we are now expecting the type
    ExpectingMainFuncArgType,

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

enum ValueType {
    Any,
    U8,
    U16,
    U32,
    U64,
    U128,
    UBIG,
    I8,
    I16,
    I32,
    I64,
    I128,
    IBIG,
    F32,
    F64,
    Decimal,
    Address,
    Asset,
}
