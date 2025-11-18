use std::fmt;

#[derive(Debug, Clone)]
pub struct Token {
    pub typ: TokenType,
    pub start: usize,
    pub length: usize,
    pub line: usize,
    pub message: String,
    pub lexeme: String,
}

impl Token {
    pub fn new(
        typ: TokenType,
        start: usize,
        length: usize,
        line: usize,
        message: String,
        lexeme: String,
    ) -> Self {
        Self {
            typ,
            start,
            length,
            line,
            message,
            lexeme,
        }
    }

    pub fn empty() -> Self {
        Token::new(TokenType::None, 0, 0, 0, String::new(), String::new())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TokenType {
    // Single character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    SemiColon,
    Colon,
    Slash,
    Star,
    // One or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literals
    Identifier,
    Str,
    Number,
    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    Val,
    Switch,
    Case,
    Default,
    While,
    Error,
    Eof,
    None,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::LeftParen => write!(f, "LEFT_PAREN"),
            Self::RightParen => write!(f, "RIGHT_PAREN"),
            Self::LeftBrace => write!(f, "LEFT_BRACE"),
            Self::RightBrace => write!(f, "RIGHT_BRACE"),
            Self::Comma => write!(f, "COMMA"),
            Self::Dot => write!(f, "DOT"),
            Self::Minus => write!(f, "MINUS"),
            Self::Plus => write!(f, "PLUS"),
            Self::SemiColon => write!(f, "SEMI_COLON"),
            Self::Colon => write!(f, "COLON"),
            Self::Slash => write!(f, "SLASH"),
            Self::Star => write!(f, "STAR"),
            Self::Bang => write!(f, "BANG"),
            Self::BangEqual => write!(f, "BANG_EQUAL"),
            Self::Equal => write!(f, "EQUAL"),
            Self::EqualEqual => write!(f, "EQUAL_EQUAL"),
            Self::Greater => write!(f, "GREATER"),
            Self::GreaterEqual => write!(f, "GREATER_EQUAL"),
            Self::Less => write!(f, "LESS"),
            Self::LessEqual => write!(f, "LESS_EQUAL"),
            Self::Identifier => write!(f, "IDENTIFIER"),
            Self::Str => write!(f, "STR"),
            Self::Number => write!(f, "NUMBER"),
            Self::And => write!(f, "AND"),
            Self::Class => write!(f, "CLASS"),
            Self::Else => write!(f, "ELSE"),
            Self::False => write!(f, "FALSE"),
            Self::Fun => write!(f, "FUN"),
            Self::For => write!(f, "FOR"),
            Self::If => write!(f, "IF"),
            Self::Nil => write!(f, "NIL"),
            Self::Or => write!(f, "OR"),
            Self::Print => write!(f, "PRINT"),
            Self::Return => write!(f, "RETURN"),
            Self::Super => write!(f, "SUPER"),
            Self::This => write!(f, "THIS"),
            Self::True => write!(f, "TRUE"),
            Self::Var => write!(f, "VAR"),
            Self::Val => write!(f, "VAL"),
            Self::Switch => write!(f, "SWITCH"),
            Self::Case => write!(f, "CASE"),
            Self::Default => write!(f, "DEFAULT"),
            Self::While => write!(f, "WHILE"),
            Self::Error => write!(f, "ERROR"),
            Self::Eof => write!(f, "EOF"),
            Self::None => write!(f, "NONE"),
        }
    }
}
