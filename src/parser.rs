use crate::core::{ASTNode, NodeKind, ParserState, TokenType};


pub fn parse(parserstate: &mut ParserState, astlist: &mut Vec<ASTNode>) {

    while parserstate.position < parserstate.tokens.len() {

        if parserstate.tokens[parserstate.position] == TokenType::SEMICOLON {
            parserstate.position += 1;
            continue;
        }

        astlist.push(parse_expression(parserstate));
    }
}



fn parse_expression(parserstate: &mut ParserState) -> ASTNode {

    let mut left = match &parserstate.tokens[parserstate.position] {

        TokenType::NUMBER(value) => ASTNode {
            kind: NodeKind::Number(value.clone()),
        },

        TokenType::IDENTIFIER(name) => ASTNode {
            kind: NodeKind::Identifier(name.clone()),
        },

        _ => panic!("Expected number or identifier"),
    };


    parserstate.position += 1;


    while parserstate.position < parserstate.tokens.len() {

        let op = match &parserstate.tokens[parserstate.position] {

            TokenType::PLUS => TokenType::PLUS,

            TokenType::MINUS => TokenType::MINUS,

            TokenType::SEMICOLON => break,

            _ => break,
        };


        parserstate.position += 1;


        let right = match &parserstate.tokens[parserstate.position] {

            TokenType::NUMBER(value) => ASTNode {
                kind: NodeKind::Number(value.clone()),
            },

            TokenType::IDENTIFIER(name) => ASTNode {
                kind: NodeKind::Identifier(name.clone()),
            },

            _ => panic!("Expected number or identifier after operator"),
        };


        parserstate.position += 1;


        left = ASTNode {
            kind: NodeKind::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            },
        };
    }


    left
}
