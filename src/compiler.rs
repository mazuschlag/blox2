use std::{env, mem};

use crate::{chunk::*, scanner::*, token::*, value::*};

#[derive(Debug, Clone)]
pub struct Compiler {
    scanner: Scanner,
    parser: Parser,
    chunk: Chunk,
}

impl Compiler {
    pub fn new(source: String) -> Self {
        Self {
            scanner: Scanner::new(source),
            parser: Parser::new(),
            chunk: Chunk::new(),
        }
    }

    pub fn compile(mut self) -> Result<Chunk, ()> {
        self.parser.reset();

        self.advance();
        self.expression();
        self.consume(TokenType::Eof, "Expect end of expression.");
        self.end();

        if self.parser.had_error {
            return Err(());
        }

        if env::var("DEBUG_TRACE_EXECUTION").is_ok_and(|var| var == "1") {
            self.chunk.disassemble("code");
        }

        Ok(self.chunk)
    }

    fn advance(&mut self) {
        self.parser.previous = mem::replace(&mut self.parser.current, Token::empty());
        loop {
            self.parser.current = self.scanner.scan_token();
            if self.parser.current.typ != TokenType::Error {
                break;
            }

            self.parser.error(None);
        }
    }

    fn consume(&mut self, typ: TokenType, message: &str) {
        if self.parser.current.typ == typ {
            self.advance();
            return;
        }

        self.parser.error(Some(message));
    }

    fn end(&mut self) {
        self.emit_byte(OpCode::Return);
    }

    fn binary(&mut self) {
        let op_type = self.parser.previous.typ;
        let rule = op_type.get_rule();
        self.parse_precedence(rule.precedence.next());
        match op_type {
            TokenType::Plus => self.emit_byte(OpCode::Add),
            TokenType::Minus => self.emit_byte(OpCode::Subtract),
            TokenType::Star => self.emit_byte(OpCode::Multiply),
            TokenType::Slash => self.emit_byte(OpCode::Divide),
            _ => panic!("Unreachable code: unknown binary operation {op_type}"),
        }
    }

    fn literal(&mut self) {
        match self.parser.previous.typ {
            TokenType::False => self.emit_byte(OpCode::False),
            TokenType::Nil => self.emit_byte(OpCode::Nil),
            TokenType::True => self.emit_byte(OpCode::True),
            _ => panic!("Unreachable code: unknown literal {}", self.parser.previous.typ),
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression");
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn number(&mut self) {
        let number = Value::Number(self.parser.previous.lexeme.parse().unwrap());
        let index = self.chunk.add_constant(number);
        self.emit_byte(OpCode::Constant(index));
    }

    fn unary(&mut self) {
        let op_type = self.parser.previous.typ;
        self.parse_precedence(Precedence::Unary);
        match op_type {
            TokenType::Minus => self.emit_byte(OpCode::Negate),
            _ => panic!("Unreachable code: unknown unary operation {op_type}"),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        match self.parser.previous.typ.get_rule().prefix {
            Some(prefix_rule) => prefix_rule(self),
            None => {
                self.parser.error(Some("Expected prefix expression."));
                return;
            }
        };

        while precedence < self.parser.current.typ.get_rule().precedence {
            self.advance();
            match self.parser.previous.typ.get_rule().infix {
                Some(infix_rule) => infix_rule(self),
                None => panic!("Unreachable code: expected infix rule"),
            };
        }
    }

    fn emit_byte(&mut self, byte: OpCode) {
        self.chunk.write(byte, self.parser.previous.line);
    }
}

#[derive(Debug, Clone)]
struct Parser {
    previous: Token,
    current: Token,
    had_error: bool,
    panic_mode: bool,
}

impl Parser {
    fn new() -> Self {
        Parser {
            previous: Token::empty(),
            current: Token::empty(),
            had_error: false,
            panic_mode: false,
        }
    }

    fn reset(&mut self) {
        self.had_error = false;
        self.panic_mode = false;
    }

    fn error(&mut self, message: Option<&str>) {
        match message {
            Some(m) => eprintln!(
                "[line {} col {} len {}] Error at '{}': {}",
                self.previous.line,
                self.previous.length,
                self.previous.start,
                self.previous.lexeme,
                m,
            ),
            None => eprintln!(
                "[line {} col {} len {}] Error at '{}': {}",
                self.current.line,
                self.current.length,
                self.current.start,
                self.current.lexeme,
                self.current.message,
            ),
        };

        self.had_error = true;
        self.panic_mode = true;
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

impl Precedence {
    pub fn next(&self) -> Self {
        match self {
            Self::None => Self::Assignment,
            Self::Assignment => Self::Or,
            Self::Or => Self::And,
            Self::And => Self::Equality,
            Self::Equality => Self::Comparison,
            Self::Comparison => Self::Term,
            Self::Term => Self::Factor,
            Self::Factor => Self::Unary,
            Self::Unary => Self::Call,
            Self::Call => Self::Primary,
            Self::Primary => Self::Primary,
        }
    }
}

struct ParseRule {
    prefix: Option<Box<dyn Fn(&mut Compiler)>>,
    infix: Option<Box<dyn Fn(&mut Compiler)>>,
    precedence: Precedence,
}

trait GetRule {
    fn get_rule(&self) -> ParseRule;
}

impl GetRule for TokenType {
    fn get_rule(&self) -> ParseRule {
        match self {
            Self::LeftParen => ParseRule {
                prefix: Some(Box::new(|compiler: &mut Compiler| compiler.grouping())),
                infix: None,
                precedence: Precedence::None,
            },
            Self::RightParen => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::LeftBrace => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::RightBrace => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::Comma => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::Dot => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::Minus => ParseRule {
                prefix: Some(Box::new(|compiler: &mut Compiler| compiler.unary())),
                infix: Some(Box::new(|compiler: &mut Compiler| compiler.binary())),
                precedence: Precedence::Term,
            },
            Self::Plus => ParseRule {
                prefix: None,
                infix: Some(Box::new(|compiler: &mut Compiler| compiler.binary())),
                precedence: Precedence::Term,
            },
            Self::Colon => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::SemiColon => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::Slash => ParseRule {
                prefix: None,
                infix: Some(Box::new(|compiler: &mut Compiler| compiler.binary())),
                precedence: Precedence::Factor,
            },
            Self::Star => ParseRule {
                prefix: None,
                infix: Some(Box::new(|compiler: &mut Compiler| compiler.binary())),
                precedence: Precedence::Factor,
            },
            Self::Bang => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::BangEqual => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::Equal => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::EqualEqual => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::Greater => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::GreaterEqual => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::Less => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::LessEqual => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::Identifier => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::Str => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::Number => ParseRule {
                prefix: Some(Box::new(|compiler: &mut Compiler| compiler.number())),
                infix: None,
                precedence: Precedence::None,
            },
            Self::And => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::Class => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::Else => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::False => ParseRule {
                prefix: Some(Box::new(|compiler: &mut Compiler| compiler.literal())),
                infix: None,
                precedence: Precedence::None,
            },
            Self::For => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::Fun => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::If => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::Nil => ParseRule {
                prefix: Some(Box::new(|compiler: &mut Compiler| compiler.literal())),
                infix: None,
                precedence: Precedence::None,
            },
            Self::Or => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::Print => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::Return => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::Super => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::This => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::True => ParseRule {
                prefix: Some(Box::new(|compiler: &mut Compiler| compiler.literal())),
                infix: None,
                precedence: Precedence::None,
            },
            Self::Var => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::Val => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::Switch => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::Case => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::Default => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::While => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::Error => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::Eof => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::None => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        }
    }
}
