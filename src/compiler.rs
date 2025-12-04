use std::{env, mem};

use crate::{arena::*, chunk::*, scanner::*, token::*, value::*};

const UNINITIALIZED_SCOPE: isize = -1;
const GLOBAL_SCOPE: usize = 0;

#[derive(Debug)]
pub struct Compiler<'a> {
    scanner: Scanner,
    parser: Parser,
    locals: Vec<Local>,
    scope_depth: usize,
    pub chunk: Chunk,
    pub objects: &'a mut Arena<Obj>,
}

impl<'a> Compiler<'a> {
    pub fn new(source: String, objects: &'a mut Arena<Obj>) -> Self {
        Self {
            scanner: Scanner::new(source),
            parser: Parser::new(),
            locals: Vec::new(),
            scope_depth: 0,
            chunk: Chunk::new(),
            objects: objects,
        }
    }

    pub fn compile(mut self) -> Result<Compiler<'a>, ()> {
        self.parser.reset();

        self.advance();
        while !self.check(TokenType::Eof) {
            self.declaration();
        }
        self.end();

        if self.parser.had_error {
            return Err(());
        }

        if env::var("DEBUG_PRINT_CODE").is_ok_and(|var| var == "1") {
            self.chunk.disassemble("code", self.objects);
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

            self.parser.error("");
        }
    }

    fn consume(&mut self, typ: TokenType, message: &str) {
        if self.parser.current.typ == typ {
            self.advance();
            return;
        }

        self.parser.error(message);
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

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;
        let mut locals = Vec::new();
        for local in &self.locals {
            if local.depth <= self.scope_depth as isize {
                locals.push(local.clone());
            } else {
                self.chunk.write(Op::Pop, self.parser.previous.line)
            }
        }

        self.locals = self.clear_scope();
    }

    fn clear_scope(&self) -> Vec<Local> {
        self.locals
            .iter()
            .take_while(|local| local.depth < self.scope_depth as isize)
            .map(|local| local.clone())
            .collect()
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

    fn block(&mut self) {
        while !self.parser.check(TokenType::RightBrace) && !self.parser.check(TokenType::Eof) {
            self.declaration();
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.");
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");

        if self.check(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_byte(Op::Nil);
        }

        self.consume(
            TokenType::SemiColon,
            "Expect ';' after variable declaration",
        );
        self.define_variable(global);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::SemiColon, "Expect ';' after expression.");
        self.emit_byte(Op::Pop);
    }

    fn if_statement(&mut self) {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");
        
        let then_jump = self.emit_jump(Op::JumpIfFalse(0));
        self.emit_byte(Op::Pop);
        self.statement();
        
        let else_jump = self.emit_jump(Op::Jump(0));
        self.patch_jump(then_jump);
        self.emit_byte(Op::Pop);
        if self.check(TokenType::Else) {
            self.statement();
        }

        self.patch_jump(else_jump);
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::SemiColon, "Expect ';' after value.");
        self.emit_byte(Op::Print);
    }

    fn synchronize(&mut self) {
        self.parser.panic_mode = false;
        while self.parser.current.typ != TokenType::Eof {
            if self.parser.previous.typ == TokenType::SemiColon {
                return;
            }

            match self.parser.current.typ {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => self.advance(),
            }
        }
    }

    fn declaration(&mut self) {
        if self.check(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.parser.panic_mode {
            self.synchronize();
        }
    }

    fn statement(&mut self) {
        if self.check(TokenType::Print) {
            self.print_statement();
        } else if self.check(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else if self.check(TokenType::If) {
            self.if_statement();
        } else {
            self.expression_statement();
        }
    }

    fn number(&mut self) {
        let lexeme = &self.parser.previous.lexeme;
        let number = Value::Number(lexeme.parse().unwrap());
        self.make_constant(number);
    }

    fn string(&mut self) {
        let lexeme = self.parser.previous.lexeme.clone();
        let string = Obj::Str(lexeme);
        self.objects.push(string);

        self.make_constant(Value::Obj(self.objects.len() - 1));
    }

    fn named_variable(&mut self, name: String, can_assign: bool) {
        let arg = self.resolve_local(&name);
        let (get_op, set_op) = match arg {
            Some(a) => (Op::GetLocal(a), Op::SetLocal(a)),
            None => {
                let arg = self.identifier_constant(name);
                (Op::GetGlobal(arg), Op::SetGlobal(arg))
            }
        };

        if can_assign && self.check(TokenType::Equal) {
            self.expression();
            self.emit_byte(set_op);
        } else {
            self.emit_byte(get_op);
        }
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(self.parser.previous.lexeme.clone(), can_assign)
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
        let can_assign = precedence <= Precedence::Assignment;
        match self.parser.previous.typ.get_rule().prefix {
            Method::Grouping(prefix_rule)
            | Method::Unary(prefix_rule)
            | Method::Number(prefix_rule)
            | Method::Str(prefix_rule)
            | Method::Literal(prefix_rule) => prefix_rule(self),
            Method::Variable(prefix_rule) => prefix_rule(self, can_assign),
            _ => {
                self.parser.error("Expected prefix expression.");
                return;
            }
        };

        while precedence <= self.parser.current.typ.get_rule().precedence {
            self.advance();
            match self.parser.previous.typ.get_rule().infix {
                Method::Binary(infix_rule) => infix_rule(self),
                _ => panic!("Unreachable code: expected infix rule"),
            };
        }

        if can_assign && self.check(TokenType::Equal) {
            self.parser.error("Invalid assignment target.");
        }
    }

    fn identifier_constant(&mut self, name: String) -> usize {
        let ident = Obj::Ident(name);
        self.objects.push(ident);
        self.chunk.add_constant(Value::Obj(self.objects.len() - 1))
    }

    fn resolve_local(&mut self, name: &String) -> Option<usize> {
        for (index, local) in self.locals.iter().enumerate().rev() {
            if &local.name == name {
                if local.depth == UNINITIALIZED_SCOPE {
                    self.parser
                        .error("Can't read local variable in its own initializer.")
                }
                return Some(index);
            }
        }

        None
    }

    fn add_local(&mut self, name: String) {
        let local = Local::new(name, UNINITIALIZED_SCOPE);
        self.locals.push(local);
    }

    fn declare_variable(&mut self) {
        if self.is_global_scope() {
            return;
        }

        let name = self.parser.previous.lexeme.clone();
        for local in self.locals.iter().rev() {
            if local.depth != UNINITIALIZED_SCOPE && local.depth < self.scope_depth as isize {
                break;
            }

            if local.name == name {
                let message = format!("Variable with name {name} already exists in this scope.");
                self.parser.error(&message);
            }
        }

        self.add_local(name);
    }

    fn parse_variable(&mut self, error_message: &str) -> usize {
        self.consume(TokenType::Identifier, error_message);
        self.declare_variable();
        if !self.is_global_scope() {
            return 0;
        }

        let name = self.parser.previous.lexeme.clone();
        self.identifier_constant(name)
    }

    fn mark_initialized(&mut self) {
        let index = self.locals.len() - 1;
        let local = &mut self.locals[index];
        local.depth = self.scope_depth as isize;
    }

    fn define_variable(&mut self, global: usize) {
        if !self.is_global_scope() {
            self.mark_initialized();
            return;
        }

        self.emit_byte(Op::DefineGlobal(global));
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.chunk.code_len();
        let op = match self.chunk.get_op(offset) {
            Op::JumpIfFalse(_) => Op::JumpIfFalse(jump),
            Op::Jump(_) => Op::Jump(jump),
            op => panic!("Op {op} at {offset} is not a valid jump op."),
        };

        *self.chunk.get_op_mut(offset) = op;
    }

    fn emit_jump(&mut self, byte: Op) -> usize {
        self.chunk.write(byte, self.parser.previous.line);
        self.chunk.code_len() - 1
    }

    fn emit_byte(&mut self, byte: Op) {
        self.chunk.write(byte, self.parser.previous.line);
    }

    fn emit_bytes(&mut self, first: Op, second: Op) {
        self.emit_byte(first);
        self.emit_byte(second);
    }

    fn make_constant(&mut self, value: Value) {
        let index = self.chunk.add_constant(value);
        self.emit_byte(Op::Constant(index));
    }

    fn is_global_scope(&self) -> bool {
        self.scope_depth == GLOBAL_SCOPE
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

    fn check(&self, typ: TokenType) -> bool {
        self.current.typ == typ
    }

    fn error(&mut self, message: &str) {
        let m = if message.is_empty() {
            &self.current.message
        } else {
            message
        };

        eprintln!(
            "[line {} col {} len {}] Error at '{}': {}",
            self.previous.line, self.previous.start, self.previous.length, self.previous.lexeme, m,
        );

        self.had_error = true;
        self.panic_mode = true;
    }
}

#[derive(Debug, Clone)]
struct Local {
    name: String,
    depth: isize,
}

impl Local {
    fn new(name: String, depth: isize) -> Local {
        Local { name, depth }
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

enum Method {
    Grouping(Box<dyn Fn(&mut Compiler)>),
    Unary(Box<dyn Fn(&mut Compiler)>),
    Binary(Box<dyn Fn(&mut Compiler)>),
    Number(Box<dyn Fn(&mut Compiler)>),
    Str(Box<dyn Fn(&mut Compiler)>),
    Literal(Box<dyn Fn(&mut Compiler)>),
    Variable(Box<dyn Fn(&mut Compiler, bool)>),
    None,
}

struct ParseRule {
    prefix: Method,
    infix: Method,
    precedence: Precedence,
}

trait GetRule {
    fn get_rule(&self) -> ParseRule;
}

impl GetRule for TokenType {
    fn get_rule(&self) -> ParseRule {
        match self {
            Self::LeftParen => ParseRule {
                prefix: Method::Grouping(Box::new(|compiler| compiler.grouping())),
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::RightParen => ParseRule {
                prefix: Method::None,
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::LeftBrace => ParseRule {
                prefix: Method::None,
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::RightBrace => ParseRule {
                prefix: Method::None,
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::Comma => ParseRule {
                prefix: Method::None,
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::Dot => ParseRule {
                prefix: Method::None,
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::Minus => ParseRule {
                prefix: Method::Unary(Box::new(|compiler| compiler.unary())),
                infix: Method::Binary(Box::new(|compiler| compiler.binary())),
                precedence: Precedence::Term,
            },
            Self::Plus => ParseRule {
                prefix: Method::None,
                infix: Method::Binary(Box::new(|compiler| compiler.binary())),
                precedence: Precedence::Term,
            },
            Self::Colon => ParseRule {
                prefix: Method::None,
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::SemiColon => ParseRule {
                prefix: Method::None,
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::Slash => ParseRule {
                prefix: Method::None,
                infix: Method::Binary(Box::new(|compiler| compiler.binary())),
                precedence: Precedence::Factor,
            },
            Self::Star => ParseRule {
                prefix: Method::None,
                infix: Method::Binary(Box::new(|compiler| compiler.binary())),
                precedence: Precedence::Factor,
            },
            Self::Bang => ParseRule {
                prefix: Method::Unary(Box::new(|compiler| compiler.unary())),
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::BangEqual => ParseRule {
                prefix: Method::None,
                infix: Method::Binary(Box::new(|compiler| compiler.binary())),
                precedence: Precedence::Equality,
            },
            Self::Equal => ParseRule {
                prefix: Method::None,
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::EqualEqual => ParseRule {
                prefix: Method::None,
                infix: Method::Binary(Box::new(|compiler| compiler.binary())),
                precedence: Precedence::Equality,
            },
            Self::Greater => ParseRule {
                prefix: Method::None,
                infix: Method::Binary(Box::new(|compiler| compiler.binary())),
                precedence: Precedence::Comparison,
            },
            Self::GreaterEqual => ParseRule {
                prefix: Method::None,
                infix: Method::Binary(Box::new(|compiler| compiler.binary())),
                precedence: Precedence::Comparison,
            },
            Self::Less => ParseRule {
                prefix: Method::None,
                infix: Method::Binary(Box::new(|compiler| compiler.binary())),
                precedence: Precedence::Comparison,
            },
            Self::LessEqual => ParseRule {
                prefix: Method::None,
                infix: Method::Binary(Box::new(|compiler| compiler.binary())),
                precedence: Precedence::Comparison,
            },
            Self::Identifier => ParseRule {
                prefix: Method::Variable(Box::new(|compiler, can_assign| {
                    compiler.variable(can_assign)
                })),
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::Str => ParseRule {
                prefix: Method::Str(Box::new(|compiler| compiler.string())),
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::Number => ParseRule {
                prefix: Method::Number(Box::new(|compiler| compiler.number())),
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::And => ParseRule {
                prefix: Method::None,
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::Class => ParseRule {
                prefix: Method::None,
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::Else => ParseRule {
                prefix: Method::None,
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::False => ParseRule {
                prefix: Method::Literal(Box::new(|compiler| compiler.literal())),
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::For => ParseRule {
                prefix: Method::None,
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::Fun => ParseRule {
                prefix: Method::None,
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::If => ParseRule {
                prefix: Method::None,
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::Nil => ParseRule {
                prefix: Method::Literal(Box::new(|compiler| compiler.literal())),
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::Or => ParseRule {
                prefix: Method::None,
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::Print => ParseRule {
                prefix: Method::None,
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::Return => ParseRule {
                prefix: Method::None,
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::Super => ParseRule {
                prefix: Method::None,
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::This => ParseRule {
                prefix: Method::None,
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::True => ParseRule {
                prefix: Method::Literal(Box::new(|compiler| compiler.literal())),
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::Var => ParseRule {
                prefix: Method::None,
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::Val => ParseRule {
                prefix: Method::None,
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::Switch => ParseRule {
                prefix: Method::None,
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::Case => ParseRule {
                prefix: Method::None,
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::Default => ParseRule {
                prefix: Method::None,
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::While => ParseRule {
                prefix: Method::None,
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::Error => ParseRule {
                prefix: Method::None,
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::Eof => ParseRule {
                prefix: Method::None,
                infix: Method::None,
                precedence: Precedence::None,
            },
            Self::None => ParseRule {
                prefix: Method::None,
                infix: Method::None,
                precedence: Precedence::None,
            },
        }
    }
}
