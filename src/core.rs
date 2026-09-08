use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Num,
    Int,
    Flag,
    Text,
    Pointer(Box<Type>),
    Array(Box<Type>, usize),
    Layout(String),
    None,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Num => write!(f, "num"),
            Type::Int => write!(f, "int"),
            Type::Flag => write!(f, "flag"),
            Type::Text => write!(f, "text"),
            Type::Pointer(t) => write!(f, "*{}", t),
            Type::Array(t, len) => write!(f, "[{}; {}]", t, len),
            Type::Layout(name) => write!(f, "layout {}", name),
            Type::None => write!(f, "none"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Number(f64),
    IntNumber(i64),
    StringLit(String),
    Identifier(String),
    Yes,
    No,

    Emit,
    When,
    Otherwise,
    During,
    Proc,
    Yield,
    Layout,
    Slot,
    Claim,
    Purge,

    TypeNum,
    TypeInt,
    TypeFlag,
    TypeText,
    TypeNone,

    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    EqualEqual,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    AmpAmp,
    PipePipe,
    Bang,
    Ampersand,
    Equal,

    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Semicolon,
    Colon,
    Comma,
    Dot,
    Arrow,

    Eof,
}

#[derive(Debug, Clone)]
pub struct ASTNode {
    pub kind: NodeKind,
}

impl ASTNode {
    pub fn new(kind: NodeKind) -> Self {
        ASTNode { kind }
    }
}

#[derive(Debug, Clone)]
pub struct LayoutField {
    pub name: String,
    pub field_type: Type,
    pub offset: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LayoutDef {
    pub name: String,
    pub fields: Vec<LayoutField>,
    pub total_size: usize,
}

#[derive(Debug, Clone)]
pub struct ProcParam {
    pub name: String,
    pub param_type: Type,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum NodeKind {
    Number(f64),
    IntNumber(i64),
    StringLit(String),
    Boolean(bool),
    Identifier(String),

    SlotDecl {
        name: String,
        slot_type: Option<Type>,
        value: Option<Box<ASTNode>>,
    },

    LayoutDecl(LayoutDef),

    ProcDecl {
        name: String,
        params: Vec<ProcParam>,
        return_type: Type,
        body: Vec<ASTNode>,
    },

    Assignment {
        target: Box<ASTNode>,
        value: Box<ASTNode>,
    },

    Emit(Box<ASTNode>),

    When {
        condition: Box<ASTNode>,
        then_branch: Vec<ASTNode>,
        else_branch: Option<Vec<ASTNode>>,
    },

    During {
        condition: Box<ASTNode>,
        body: Vec<ASTNode>,
    },

    Yield(Option<Box<ASTNode>>),
    Claim(Box<ASTNode>),
    Purge(Box<ASTNode>),

    BinaryOp {
        left: Box<ASTNode>,
        op: TokenType,
        right: Box<ASTNode>,
    },

    UnaryOp {
        op: TokenType,
        operand: Box<ASTNode>,
    },

    Call {
        callee: String,
        args: Vec<ASTNode>,
    },

    AddressOf(Box<ASTNode>),
    Deref(Box<ASTNode>),
    FieldAccess {
        object: Box<ASTNode>,
        field: String,
    },
    ArrayIndex {
        array: Box<ASTNode>,
        index: Box<ASTNode>,
    },
    ArrayLiteral(Vec<ASTNode>),
    Block(Vec<ASTNode>),
    ExprStatement(Box<ASTNode>),
}

pub struct ParserState {
    tokens: Vec<TokenType>,
    position: usize,
}

impl ParserState {
    pub fn new(tokens: Vec<TokenType>) -> Self {
        ParserState { tokens, position: 0 }
    }

    pub fn peek(&self) -> Option<&TokenType> {
        self.tokens.get(self.position)
    }

    pub fn advance(&mut self) -> Option<TokenType> {
        if self.position < self.tokens.len() {
            let tok = self.tokens[self.position].clone();
            self.position += 1;
            Some(tok)
        } else {
            None
        }
    }

    pub fn consume(&mut self, expected: &TokenType) {
        if let Some(tok) = self.peek() {
            if std::mem::discriminant(tok) == std::mem::discriminant(expected) {
                self.advance();
                return;
            }
        }
        panic!("Syntax error: expected {:?}, got {:?}", expected, self.peek());
    }
}