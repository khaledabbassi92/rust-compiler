use crate::core::{TokenType, ParserState, NodeKind};

pub fn parse(parserstate: &mut ParserState, astlist: &mut Vec<NodeKind>) {
    // Main loop: Process the entire list of tokens
    while parserstate.position < parserstate.tokens.len() {
        
        // --- Inlined parse_expression logic ---
        // Get the first number (the "left" side)
        let token = &parserstate.tokens[parserstate.position];
        let mut left = match token.kind {
            TokenType::NUMBER => NodeKind::Number(token.value),
            _ => panic!("Expected number"),
        };
        parserstate.position += 1;

        // Loop while there is an operator ahead
        while parserstate.position < parserstate.tokens.len() {
            let next_token = &parserstate.tokens[parserstate.position];
            
            if next_token.kind == TokenType::PLUS || next_token.kind == TokenType::MINUS {
                let op = next_token.clone();
                parserstate.position += 1; // Consume operator
                
                // Get the next number (the "right" side)
                let right_token = &parserstate.tokens[parserstate.position];
                let right = match right_token.kind {
                    TokenType::NUMBER => NodeKind::Number(right_token.value),
                    _ => panic!("Expected number after operator"),
                };
                parserstate.position += 1; // Consume right number
                
                // Wrap left and right into a new node
                left = NodeKind::BinaryOp {
                    left: Box::new(left),
                    op: op.kind,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        astlist.push(left);
    }
}