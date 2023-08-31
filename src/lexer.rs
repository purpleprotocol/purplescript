use std::iter::Peekable;
use std::str::Chars;

pub fn tokenize(input: &str) -> Tokens {
    Tokens::new(input)
}

#[derive(Debug, PartialEq, Eq)]
pub enum Symbol {
    Ampersand,
    Asterisk,
    BraceLeft,
    BraceRight,
    BracketLeft,
    BracketRight,
    Equal,
    Minus,
    ParenthesisLeft,
    ParenthesisRight,
    Plus,
    Colon,
    Semicolon,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Keyword {
    Function,
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
    As,
    Else,
    For,
    If,
    Let,
    Return,
    Revert,
    While,
    Malleable,
    Asset,
    Address,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenKind {
    Identifier(String),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    Keyword(Keyword),
    String(String),
    Symbol(Symbol),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Position {
    pub column: usize,
    pub line: usize,
}

impl Position {
    fn new(column: usize, line: usize) -> Self {
        Self { column, line }
    }
}

impl Default for Position {
    fn default() -> Position {
        Position::new(1, 1)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub position: Position,
}

impl Token {
    fn new(kind: TokenKind, position: Position) -> Self {
        Self { kind, position }
    }
}

pub struct Tokens<'a> {
    chars: Peekable<Chars<'a>>,
    position: Position,
}

impl<'a> Tokens<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
            position: Position::default(),
        }
    }

    fn consume_number_literal(&mut self) -> u32 {
        let mut number = 0;
        while let Some(&character) = self.chars.peek() {
            if character.is_ascii_digit() {
                self.consume_character();
                number = number * 10 + character.to_digit(10).unwrap();
            } else {
                break;
            }
        }
        number
    }

    fn consume_character_literal(&mut self) -> char {
        self.consume_character();
        let character = self.consume_character();
        self.consume_character();
        character.unwrap()
    }

    fn consume_string_literal(&mut self, c: char) -> String {
        let mut string = String::new();
        self.consume_character();
        while let Some(character) = self.consume_character() {
            if character == c {
                break;
            } else {
                string.push(character);
            }
        }
        string
    }

    fn consume_identifier(&mut self) -> String {
        let mut string = String::new();
        while let Some(&character) = self.chars.peek() {
            if character.is_ascii_alphanumeric() {
                string.push(character);
                self.consume_character();
            } else {
                break;
            }
        }
        string
    }

    fn consume_whitespaces(&mut self) {
        while let Some(&character) = self.chars.peek() {
            if character == '\n' {
                self.consume_new_line();
            } else if character.is_whitespace() {
                self.consume_character();
            } else {
                break;
            }
        }
    }

    fn consume_new_line(&mut self) {
        self.position.column = 1;
        self.position.line += 1;
        self.chars.next();
    }

    fn consume_character(&mut self) -> Option<char> {
        self.position.column += 1;
        self.chars.next()
    }
}

impl Iterator for Tokens<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.consume_whitespaces();
        let mut token = None;
        if let Some(&character) = self.chars.peek() {
            let position = self.position.clone();
            match character {
                ':' => {
                    self.consume_character();
                    token = Some(Token::new(TokenKind::Symbol(Symbol::Colon), position));
                }
                ';' => {
                    self.consume_character();
                    token = Some(Token::new(TokenKind::Symbol(Symbol::Semicolon), position));
                }
                '+' => {
                    self.consume_character();
                    token = Some(Token::new(TokenKind::Symbol(Symbol::Plus), position));
                }
                '-' => {
                    self.consume_character();
                    token = Some(Token::new(TokenKind::Symbol(Symbol::Minus), position));
                }
                '=' => {
                    self.consume_character();
                    token = Some(Token::new(TokenKind::Symbol(Symbol::Equal), position));
                }
                '&' => {
                    self.consume_character();
                    token = Some(Token::new(TokenKind::Symbol(Symbol::Ampersand), position));
                }
                '*' => {
                    self.consume_character();
                    token = Some(Token::new(TokenKind::Symbol(Symbol::Asterisk), position));
                }
                '(' => {
                    self.consume_character();
                    token = Some(Token::new(
                        TokenKind::Symbol(Symbol::ParenthesisLeft),
                        position,
                    ));
                }
                ')' => {
                    self.consume_character();
                    token = Some(Token::new(
                        TokenKind::Symbol(Symbol::ParenthesisRight),
                        position,
                    ));
                }
                '{' => {
                    self.consume_character();
                    token = Some(Token::new(TokenKind::Symbol(Symbol::BraceLeft), position));
                }
                '}' => {
                    self.consume_character();
                    token = Some(Token::new(TokenKind::Symbol(Symbol::BraceRight), position));
                }
                '[' => {
                    self.consume_character();
                    token = Some(Token::new(TokenKind::Symbol(Symbol::BracketLeft), position));
                }
                ']' => {
                    self.consume_character();
                    token = Some(Token::new(
                        TokenKind::Symbol(Symbol::BracketRight),
                        position,
                    ));
                }
                '"' => {
                    let value = self.consume_string_literal('"');
                    token = Some(Token::new(TokenKind::String(value), position));
                }
                '\'' => {
                    let value = self.consume_string_literal('\'');
                    token = Some(Token::new(TokenKind::String(value), position));
                }
                _ => {
                    if character.is_ascii_digit() {
                        token = Some(Token::new(
                            TokenKind::U32(self.consume_number_literal()),
                            position,
                        ));
                    } else if character.is_ascii_alphabetic() {
                        let name = self.consume_identifier();
                        match &*name {
                            "function" => {
                                token = Some(Token::new(TokenKind::Keyword(Keyword::Function), position));
                            }
                            "let" => {
                                token = Some(Token::new(TokenKind::Keyword(Keyword::Let), position));
                            }
                            "u8" => {
                                token = Some(Token::new(TokenKind::Keyword(Keyword::U8), position));
                            }
                            "u16" => {
                                token = Some(Token::new(TokenKind::Keyword(Keyword::U16), position));
                            }
                            "u32" => {
                                token = Some(Token::new(TokenKind::Keyword(Keyword::U32), position));
                            }
                            "u64" => {
                                token = Some(Token::new(TokenKind::Keyword(Keyword::U64), position));
                            }
                            "u128" => {
                                token = Some(Token::new(TokenKind::Keyword(Keyword::U128), position));
                            }
                            "ubig" => {
                                token = Some(Token::new(TokenKind::Keyword(Keyword::UBIG), position));
                            }
                            "i8" => {
                                token = Some(Token::new(TokenKind::Keyword(Keyword::I8), position));
                            }
                            "i16" => {
                                token = Some(Token::new(TokenKind::Keyword(Keyword::I16), position));
                            }
                            "i32" => {
                                token = Some(Token::new(TokenKind::Keyword(Keyword::I32), position));
                            }
                            "i64" => {
                                token = Some(Token::new(TokenKind::Keyword(Keyword::I64), position));
                            }
                            "i128" => {
                                token = Some(Token::new(TokenKind::Keyword(Keyword::I128), position));
                            }
                            "ibig" => {
                                token = Some(Token::new(TokenKind::Keyword(Keyword::IBIG), position));
                            }
                            "f32" => {
                                token = Some(Token::new(TokenKind::Keyword(Keyword::F32), position));
                            }
                            "f64" => {
                                token = Some(Token::new(TokenKind::Keyword(Keyword::F64), position));
                            }
                            "decimal" => {
                                token = Some(Token::new(TokenKind::Keyword(Keyword::Decimal), position));
                            }
                            "malleable" => {
                                token = Some(Token::new(TokenKind::Keyword(Keyword::Malleable), position));
                            }
                            "asset" => {
                                token = Some(Token::new(TokenKind::Keyword(Keyword::Asset), position));
                            }
                            "address" => {
                                token = Some(Token::new(TokenKind::Keyword(Keyword::Address), position));
                            }
                            "revert" => {
                                token =
                                    Some(Token::new(TokenKind::Keyword(Keyword::Revert), position));
                            }
                            "if" => {
                                token = Some(Token::new(TokenKind::Keyword(Keyword::If), position));
                            }
                            "as" => {
                                token =
                                    Some(Token::new(TokenKind::Keyword(Keyword::As), position))
                            }
                            "else" => {
                                token =
                                    Some(Token::new(TokenKind::Keyword(Keyword::Else), position));
                            }
                            "return" => {
                                token =
                                    Some(Token::new(TokenKind::Keyword(Keyword::Return), position));
                            }
                            "while" => {
                                token =
                                    Some(Token::new(TokenKind::Keyword(Keyword::While), position));
                            }
                            "for" => {
                                token =
                                    Some(Token::new(TokenKind::Keyword(Keyword::For), position));
                            }
                            _ => {
                                token = Some(Token::new(TokenKind::Identifier(name), position));
                            }
                        }
                    }
                }
            }
        }
        token
    }
}

#[cfg(test)]
mod tests {
    use super::{tokenize, Keyword, Position, Symbol, Token, TokenKind};

    #[test]
    fn test_tokenize_empty_string() {
        let tokens: Vec<Token> = tokenize("").collect();
        assert_eq!(tokens, vec![]);
    }

    #[test]
    fn test_tokenize_whitespace() {
        let tokens: Vec<Token> = tokenize(" ").collect();
        assert_eq!(tokens, vec![]);
    }

    #[test]
    fn test_tokenize_integer_and_plus_and_minus() {
        let tokens: Vec<Token> = tokenize("1 + 2 - 3").collect();
        assert_eq!(
            tokens,
            vec![
                Token::new(TokenKind::U32(1), Position::new(1, 1)),
                Token::new(TokenKind::Symbol(Symbol::Plus), Position::new(3, 1)),
                Token::new(TokenKind::U32(2), Position::new(5, 1)),
                Token::new(TokenKind::Symbol(Symbol::Minus), Position::new(7, 1)),
                Token::new(TokenKind::U32(3), Position::new(9, 1)),
            ]
        );
    }

    #[test]
    fn test_tokenize_string() {
        let tokens: Vec<Token> = tokenize("\"dummy\"").collect();
        assert_eq!(
            tokens,
            vec![Token::new(
                TokenKind::String("dummy".to_string()),
                Position::new(1, 1)
            ),]
        );
    }

    #[test]
    fn test_tokenize_string_2() {
        let tokens: Vec<Token> = tokenize("'dummy'").collect();
        assert_eq!(
            tokens,
            vec![Token::new(
                TokenKind::String("dummy".to_string()),
                Position::new(1, 1)
            ),]
        );
    }

    #[test]
    fn test_tokenize_assign() {
        let tokens: Vec<Token> = tokenize("a = 1").collect();
        assert_eq!(
            tokens,
            vec![
                Token::new(TokenKind::Identifier("a".to_string()), Position::new(1, 1)),
                Token::new(TokenKind::Symbol(Symbol::Equal), Position::new(3, 1)),
                Token::new(TokenKind::U32(1), Position::new(5, 1)),
            ]
        )
    }

    #[test]
    fn test_tokenize_if() {
        let tokens: Vec<Token> = tokenize("if (1) { 2 } else { 3 }").collect();
        assert_eq!(
            tokens,
            vec![
                Token::new(TokenKind::Keyword(Keyword::If), Position::new(1, 1)),
                Token::new(
                    TokenKind::Symbol(Symbol::ParenthesisLeft),
                    Position::new(4, 1)
                ),
                Token::new(TokenKind::U32(1), Position::new(5, 1)),
                Token::new(
                    TokenKind::Symbol(Symbol::ParenthesisRight),
                    Position::new(6, 1)
                ),
                Token::new(TokenKind::Symbol(Symbol::BraceLeft), Position::new(8, 1)),
                Token::new(TokenKind::U32(2), Position::new(10, 1)),
                Token::new(TokenKind::Symbol(Symbol::BraceRight), Position::new(12, 1)),
                Token::new(TokenKind::Keyword(Keyword::Else), Position::new(14, 1)),
                Token::new(TokenKind::Symbol(Symbol::BraceLeft), Position::new(19, 1)),
                Token::new(TokenKind::U32(3), Position::new(21, 1)),
                Token::new(TokenKind::Symbol(Symbol::BraceRight), Position::new(23, 1)),
            ]
        )
    }

    #[test]
    fn test_tokenize_while() {
        let tokens: Vec<Token> = tokenize("while (1) return 2;").collect();
        assert_eq!(
            tokens,
            vec![
                Token::new(TokenKind::Keyword(Keyword::While), Position::new(1, 1)),
                Token::new(
                    TokenKind::Symbol(Symbol::ParenthesisLeft),
                    Position::new(7, 1)
                ),
                Token::new(TokenKind::U32(1), Position::new(8, 1)),
                Token::new(
                    TokenKind::Symbol(Symbol::ParenthesisRight),
                    Position::new(9, 1)
                ),
                Token::new(TokenKind::Keyword(Keyword::Return), Position::new(11, 1)),
                Token::new(TokenKind::U32(2), Position::new(18, 1)),
                Token::new(TokenKind::Symbol(Symbol::Semicolon), Position::new(19, 1)),
            ]
        )
    }

    #[test]
    fn test_tokenize_new_line() {
        let tokens: Vec<Token> = tokenize("1\n;").collect();
        assert_eq!(
            tokens,
            vec![
                Token::new(TokenKind::U32(1), Position::new(1, 1)),
                Token::new(TokenKind::Symbol(Symbol::Semicolon), Position::new(1, 2)),
            ]
        );
    }

    #[test]
    fn test_tokenize_for() {
        let tokens: Vec<Token> = tokenize("for (i = 10; i; i = i - 1) 2;").collect();
        assert_eq!(
            tokens,
            vec![
                Token::new(TokenKind::Keyword(Keyword::For), Position::new(1, 1)),
                Token::new(
                    TokenKind::Symbol(Symbol::ParenthesisLeft),
                    Position::new(5, 1)
                ),
                Token::new(TokenKind::Identifier("i".to_string()), Position::new(6, 1)),
                Token::new(TokenKind::Symbol(Symbol::Equal), Position::new(8, 1)),
                Token::new(TokenKind::U32(10), Position::new(10, 1)),
                Token::new(TokenKind::Symbol(Symbol::Semicolon), Position::new(12, 1)),
                Token::new(TokenKind::Identifier("i".to_string()), Position::new(14, 1)),
                Token::new(TokenKind::Symbol(Symbol::Semicolon), Position::new(15, 1)),
                Token::new(TokenKind::Identifier("i".to_string()), Position::new(17, 1)),
                Token::new(TokenKind::Symbol(Symbol::Equal), Position::new(19, 1)),
                Token::new(TokenKind::Identifier("i".to_string()), Position::new(21, 1)),
                Token::new(TokenKind::Symbol(Symbol::Minus), Position::new(23, 1)),
                Token::new(TokenKind::U32(1), Position::new(25, 1)),
                Token::new(
                    TokenKind::Symbol(Symbol::ParenthesisRight),
                    Position::new(26, 1)
                ),
                Token::new(TokenKind::U32(2), Position::new(28, 1)),
                Token::new(TokenKind::Symbol(Symbol::Semicolon), Position::new(29, 1)),
            ]
        )
    }

    #[test]
    fn test_tokenize_integer_declaration() {
        let tokens: Vec<Token> = tokenize("let a: u16 = 54354;").collect();
        assert_eq!(
            tokens,
            vec![
                Token::new(TokenKind::Keyword(Keyword::Let), Position::new(1, 1)),
                Token::new(TokenKind::Identifier("a".to_string()), Position::new(5, 1)),
                Token::new(TokenKind::Symbol(Symbol::Colon), Position::new(6, 1)),
                Token::new(TokenKind::Keyword(Keyword::U16), Position::new(8, 1)),
                Token::new(TokenKind::Symbol(Symbol::Equal), Position::new(12, 1)),
                Token::new(TokenKind::U16(54354), Position::new(14, 1)),
                Token::new(TokenKind::Symbol(Symbol::Semicolon), Position::new(15, 1)),
            ]
        );
    }

    #[test]
    fn test_tokenize_array() {
        let tokens: Vec<Token> = tokenize("a[0]").collect();
        assert_eq!(
            tokens,
            vec![
                Token::new(TokenKind::Identifier("a".to_string()), Position::new(1, 1)),
                Token::new(TokenKind::Symbol(Symbol::BracketLeft), Position::new(2, 1)),
                Token::new(TokenKind::U32(0), Position::new(3, 1)),
                Token::new(TokenKind::Symbol(Symbol::BracketRight), Position::new(4, 1)),
            ]
        );
    }
}