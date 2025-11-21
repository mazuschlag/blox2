use std::{env, mem, rc::Rc};

use crate::{chunk::*, scanner::*, token::*, value::*};

#[derive(Debug, Clone)]
pub struct Compiler {
    scanner: Scanner,
    parser: Parser,
    pub chunk: Chunk,
    pub objects: Rc<Obj>,
}

impl Compiler {
    pub fn new(source: String, objects: Rc<Obj>) -> Self {
        Self {
            scanner: Scanner::new(source),
            parser: Parser::new(),
            chunk: Chunk::new(),
            objects: objects,
        }
    }

    pub fn compile(mut self) -> Result<Compiler, ()> {
        self.parser.reset();

        self.advance();
        while !self.check(TokenType::Eof) {
            self.declaration();
        }
        self.end();

        if self.parser.had_error {
            return Err(());
        }

        if env::var("DEBUG_TRACE_EXECUTION").is_ok_and(|var| var == "1") {
            self.chunk.disassemble("code");
        }

        Ok(self)
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

    fn check(&mut self, typ: TokenType) -> bool {
        if !self.parser.check(typ) {
            return false;
        }

        self.advance();
        true
    }

    fn end(&mut self) {
        self.emit_byte(Op::Return);
    }

    fn binary(&mut self) {
        let op_type = self.parser.previous.typ;
        let rule = op_type.get_rule();
        self.parse_precedence(rule.precedence.next());
        match op_type {
            TokenType::Plus => self.emit_byte(Op::Add),
            TokenType::Minus => self.emit_byte(Op::Subtract),
            TokenType::Star => self.emit_byte(Op::Multiply),
            TokenType::Slash => self.emit_byte(Op::Divide),
            TokenType::BangEqual => self.emit_bytes(Op::Equal, Op::Not),
            TokenType::EqualEqual => self.emit_byte(Op::Equal),
            TokenType::Greater => self.emit_byte(Op::Greater),
            TokenType::GreaterEqual => self.emit_bytes(Op::Less, Op::Not),
            TokenType::Less => self.emit_byte(Op::Less),
            TokenType::LessEqual => self.emit_bytes(Op::Greater, Op::Not),
            _ => panic!("Unreachable code: unknown binary operation {op_type}"),
        }
    }

    fn literal(&mut self) {
        match self.parser.previous.typ {
            TokenType::False => self.emit_byte(Op::False),
            TokenType::Nil => self.emit_byte(Op::Nil),
            TokenType::True => self.emit_byte(Op::True),
            _ => panic!(
                "Unreachable code: unknown literal {}",
                self.parser.previous.typ
            ),
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression");
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::SemiColon, "Expect ';' after value.");
        self.emit_byte(Op::Print);
    }

    fn declaration(&mut self) {
        self.statement();
    }

    fn statement(&mut self) {
        if self.check(TokenType::Print) {
            self.print_statement();
        }
    }

    fn number(&mut self) {
        let lexeme = &self.parser.previous.lexeme;
        let number = Value::Number(lexeme.parse().unwrap());
        self.emit_constant(number);
    }

    fn string(&mut self) {
        let lexeme = &self.parser.previous.lexeme;
        let string = Rc::new(Obj::Str(Rc::clone(&self.objects), lexeme.clone()));
        self.objects = Rc::clone(&string);
        self.emit_constant(Value::Obj(string));
    }

    fn unary(&mut self) {
        let op_type = self.parser.previous.typ;
        self.parse_precedence(Precedence::Unary);
        match op_type {
            TokenType::Bang => self.emit_byte(Op::Not),
            TokenType::Minus => self.emit_byte(Op::Negate),
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

    fn emit_byte(&mut self, byte: Op) {
        self.chunk.write(byte, self.parser.previous.line);
    }

    fn emit_bytes(&mut self, first: Op, second: Op) {
        self.emit_byte(first);
        self.emit_byte(second);
    }

    fn emit_constant(&mut self, value: Value) {
        let index = self.chunk.add_constant(value);
        self.emit_byte(Op::Constant(index));
    }

    fn get_method(typ: Method) -> CompilerMethod {
        match typ {
            Method::Grouping => Box::new(|compiler| compiler.grouping()),
            Method::Unary => Box::new(|compiler| compiler.unary()),
            Method::Binary => Box::new(|compiler| compiler.binary()),
            Method::Number => Box::new(|compiler| compiler.number()),
            Method::Str => Box::new(|compiler| compiler.string()),
            Method::Literal => Box::new(|compiler| compiler.literal()),
        }
    }
}

type CompilerMethod = Box<dyn Fn(&mut Compiler)>;

#[derive(Debug, Clone, Copy)]
enum Method {
    Grouping,
    Unary,
    Binary,
    Number,
    Str,
    Literal,
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

    fn check(&self, typ: TokenType) -> bool {
        self.current.typ == typ
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
    prefix: Option<CompilerMethod>,
    infix: Option<CompilerMethod>,
    precedence: Precedence,
}

trait GetRule {
    fn get_rule(&self) -> ParseRule;
}

impl GetRule for TokenType {
    fn get_rule(&self) -> ParseRule {
        match self {
            Self::LeftParen => ParseRule {
                prefix: Some(Compiler::get_method(Method::Grouping)),
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
                prefix: Some(Compiler::get_method(Method::Unary)),
                infix: Some(Compiler::get_method(Method::Binary)),
                precedence: Precedence::Term,
            },
            Self::Plus => ParseRule {
                prefix: None,
                infix: Some(Compiler::get_method(Method::Binary)),
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
                infix: Some(Compiler::get_method(Method::Binary)),
                precedence: Precedence::Factor,
            },
            Self::Star => ParseRule {
                prefix: None,
                infix: Some(Compiler::get_method(Method::Binary)),
                precedence: Precedence::Factor,
            },
            Self::Bang => ParseRule {
                prefix: Some(Compiler::get_method(Method::Unary)),
                infix: None,
                precedence: Precedence::None,
            },
            Self::BangEqual => ParseRule {
                prefix: None,
                infix: Some(Compiler::get_method(Method::Binary)),
                precedence: Precedence::Equality,
            },
            Self::Equal => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::EqualEqual => ParseRule {
                prefix: None,
                infix: Some(Compiler::get_method(Method::Binary)),
                precedence: Precedence::Equality,
            },
            Self::Greater => ParseRule {
                prefix: None,
                infix: Some(Compiler::get_method(Method::Binary)),
                precedence: Precedence::Comparison,
            },
            Self::GreaterEqual => ParseRule {
                prefix: None,
                infix: Some(Compiler::get_method(Method::Binary)),
                precedence: Precedence::Comparison,
            },
            Self::Less => ParseRule {
                prefix: None,
                infix: Some(Compiler::get_method(Method::Binary)),
                precedence: Precedence::Comparison,
            },
            Self::LessEqual => ParseRule {
                prefix: None,
                infix: Some(Compiler::get_method(Method::Binary)),
                precedence: Precedence::Comparison,
            },
            Self::Identifier => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Self::Str => ParseRule {
                prefix: Some(Compiler::get_method(Method::Str)),
                infix: None,
                precedence: Precedence::None,
            },
            Self::Number => ParseRule {
                prefix: Some(Compiler::get_method(Method::Number)),
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
                prefix: Some(Compiler::get_method(Method::Literal)),
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
                prefix: Some(Compiler::get_method(Method::Literal)),
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
                prefix: Some(Compiler::get_method(Method::Literal)),
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
