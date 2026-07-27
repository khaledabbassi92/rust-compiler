

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType{
	KEYWORD(String),
	IDENTIFIER(String),
	LEFTPAR,
	RIGHTPAR,
	EQ,
	PLUS,
	MINUS,
	SEMICOLON,
	NUMBER(String),	
}

#[derive(Debug)]
pub struct Text {
	pub source: String,
	pub current: usize,
	pub length: usize,
}

pub struct ParserState{
	
	pub tokens : Vec<TokenType>,
	pub position : usize,

}

#[derive(Debug, Clone)]
pub struct ASTNode {
	
	pub kind: NodeKind,
	
}


#[derive(Debug, Clone)]
pub enum NodeKind{
	Number(String),
	Identifier(String),
	
	BinaryOp{
		left: Box<ASTNode>,
		op: TokenType,
		right: Box<ASTNode>,
		
	},
	
	
	
	
}
