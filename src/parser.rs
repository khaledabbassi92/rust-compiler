use crate::core::{ASTNode, LayoutDef, LayoutField, NodeKind, ParserState, ProcParam, TokenType, Type};

pub struct Parser {
    state: ParserState,
}

impl Parser {
    pub fn new(tokens: Vec<TokenType>) -> Self {
        Parser {
            state: ParserState::new(tokens),
        }
    }

    pub fn parse_program(&mut self) -> Vec<ASTNode> {
        let mut nodes = Vec::new();
        while self.state.peek() != Some(&TokenType::Eof) && self.state.peek().is_some() {
            nodes.push(self.parse_declaration_or_statement());
        }
        nodes
    }

    fn parse_type(&mut self) -> Type {
        match self.state.advance() {
            Some(TokenType::TypeNum) => Type::Num,
            Some(TokenType::TypeInt) => Type::Int,
            Some(TokenType::TypeFlag) => Type::Flag,
            Some(TokenType::TypeText) => Type::Text,
            Some(TokenType::TypeNone) => Type::None,
            Some(TokenType::Star) => {
                let inner = self.parse_type();
                Type::Pointer(Box::new(inner))
            }
            Some(TokenType::LeftBracket) => {
                let inner = self.parse_type();
                self.state.consume(&TokenType::Semicolon);
                let len = match self.state.advance() {
                    Some(TokenType::IntNumber(n)) => n as usize,
                    other => panic!("Expected array length integer, found {:?}", other),
                };
                self.state.consume(&TokenType::RightBracket);
                Type::Array(Box::new(inner), len)
            }
            Some(TokenType::Layout) => {
                match self.state.advance() {
                    Some(TokenType::Identifier(name)) => Type::Layout(name),
                    other => panic!("Expected layout name, got {:?}", other),
                }
            }
            Some(TokenType::Identifier(name)) => Type::Layout(name),
            other => panic!("Expected type name, got {:?}", other),
        }
    }

    fn parse_declaration_or_statement(&mut self) -> ASTNode {
        match self.state.peek() {
            Some(TokenType::Layout) => self.parse_layout_decl(),
            Some(TokenType::Proc) => self.parse_proc_decl(),
            Some(TokenType::Slot) => self.parse_slot_decl(),
            Some(TokenType::Emit) => self.parse_emit(),
            Some(TokenType::When) => self.parse_when(),
            Some(TokenType::During) => self.parse_during(),
            Some(TokenType::Yield) => self.parse_yield(),
            Some(TokenType::LeftBrace) => {
                let stmts = self.parse_block_statements();
                ASTNode::new(NodeKind::Block(stmts))
            }
            _ => self.parse_assignment_or_expr_stmt(),
        }
    }

    fn parse_layout_decl(&mut self) -> ASTNode {
        self.state.consume(&TokenType::Layout);
        let name = match self.state.advance() {
            Some(TokenType::Identifier(n)) => n,
            other => panic!("Expected layout identifier, got {:?}", other),
        };

        self.state.consume(&TokenType::LeftBrace);
        let mut fields = Vec::new();
        let mut offset = 0;

        while self.state.peek() != Some(&TokenType::RightBrace) {
            let field_name = match self.state.advance() {
                Some(TokenType::Identifier(fn_name)) => fn_name,
                other => panic!("Expected field name, got {:?}", other),
            };
            self.state.consume(&TokenType::Colon);
            let field_type = self.parse_type();

            fields.push(LayoutField {
                name: field_name,
                field_type,
                offset,
            });
            offset += 8; // 64-bit aligned

            if self.state.peek() == Some(&TokenType::Comma) {
                self.state.advance();
            }
        }
        self.state.consume(&TokenType::RightBrace);

        ASTNode::new(NodeKind::LayoutDecl(LayoutDef {
            name,
            fields,
            total_size: offset,
        }))
    }

    fn parse_proc_decl(&mut self) -> ASTNode {
        self.state.consume(&TokenType::Proc);
        let name = match self.state.advance() {
            Some(TokenType::Identifier(n)) => n,
            other => panic!("Expected procedure name, got {:?}", other),
        };

        self.state.consume(&TokenType::LeftParen);
        let mut params = Vec::new();
        while self.state.peek() != Some(&TokenType::RightParen) {
            let p_name = match self.state.advance() {
                Some(TokenType::Identifier(pn)) => pn,
                other => panic!("Expected parameter name, got {:?}", other),
            };
            self.state.consume(&TokenType::Colon);
            let p_type = self.parse_type();
            params.push(ProcParam { name: p_name, param_type: p_type });

            if self.state.peek() == Some(&TokenType::Comma) {
                self.state.advance();
            }
        }
        self.state.consume(&TokenType::RightParen);

        let return_type = if self.state.peek() == Some(&TokenType::Arrow) {
            self.state.advance();
            self.parse_type()
        } else {
            Type::None
        };

        let body = self.parse_block_statements();

        ASTNode::new(NodeKind::ProcDecl {
            name,
            params,
            return_type,
            body,
        })
    }

    fn parse_slot_decl(&mut self) -> ASTNode {
        self.state.consume(&TokenType::Slot);
        let name = match self.state.advance() {
            Some(TokenType::Identifier(n)) => n,
            other => panic!("Expected slot variable name, got {:?}", other),
        };

        let slot_type = if self.state.peek() == Some(&TokenType::Colon) {
            self.state.advance();
            Some(self.parse_type())
        } else {
            None
        };

        let value = if self.state.peek() == Some(&TokenType::Equal) {
            self.state.advance();
            Some(Box::new(self.parse_expression()))
        } else {
            None
        };

        self.state.consume(&TokenType::Semicolon);
        ASTNode::new(NodeKind::SlotDecl { name, slot_type, value })
    }

    fn parse_emit(&mut self) -> ASTNode {
        self.state.consume(&TokenType::Emit);
        let expr = self.parse_expression();
        self.state.consume(&TokenType::Semicolon);
        ASTNode::new(NodeKind::Emit(Box::new(expr)))
    }

    fn parse_when(&mut self) -> ASTNode {
        self.state.consume(&TokenType::When);
        let condition = self.parse_expression();
        let then_branch = self.parse_block_statements();

        let else_branch = if self.state.peek() == Some(&TokenType::Otherwise) {
            self.state.advance();
            if self.state.peek() == Some(&TokenType::When) {
                Some(vec![self.parse_when()])
            } else {
                Some(self.parse_block_statements())
            }
        } else {
            None
        };

        ASTNode::new(NodeKind::When {
            condition: Box::new(condition),
            then_branch,
            else_branch,
        })
    }

    fn parse_during(&mut self) -> ASTNode {
        self.state.consume(&TokenType::During);
        let condition = self.parse_expression();
        let body = self.parse_block_statements();
        ASTNode::new(NodeKind::During {
            condition: Box::new(condition),
            body,
        })
    }

    fn parse_yield(&mut self) -> ASTNode {
        self.state.consume(&TokenType::Yield);
        let val = if self.state.peek() != Some(&TokenType::Semicolon) {
            Some(Box::new(self.parse_expression()))
        } else {
            None
        };
        self.state.consume(&TokenType::Semicolon);
        ASTNode::new(NodeKind::Yield(val))
    }

    fn parse_block_statements(&mut self) -> Vec<ASTNode> {
        self.state.consume(&TokenType::LeftBrace);
        let mut stmts = Vec::new();
        while self.state.peek() != Some(&TokenType::RightBrace) && self.state.peek().is_some() {
            stmts.push(self.parse_declaration_or_statement());
        }
        self.state.consume(&TokenType::RightBrace);
        stmts
    }

    fn parse_assignment_or_expr_stmt(&mut self) -> ASTNode {
        let expr = self.parse_expression();

        if self.state.peek() == Some(&TokenType::Equal) {
            self.state.advance();
            let value = self.parse_expression();
            self.state.consume(&TokenType::Semicolon);
            ASTNode::new(NodeKind::Assignment {
                target: Box::new(expr),
                value: Box::new(value),
            })
        } else {
            self.state.consume(&TokenType::Semicolon);
            ASTNode::new(NodeKind::ExprStatement(Box::new(expr)))
        }
    }

    pub fn parse_expression(&mut self) -> ASTNode {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> ASTNode {
        let mut left = self.parse_logical_and();
        while self.state.peek() == Some(&TokenType::PipePipe) {
            let op = self.state.advance().unwrap();
            let right = self.parse_logical_and();
            left = ASTNode::new(NodeKind::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            });
        }
        left
    }

    fn parse_logical_and(&mut self) -> ASTNode {
        let mut left = self.parse_equality();
        while self.state.peek() == Some(&TokenType::AmpAmp) {
            let op = self.state.advance().unwrap();
            let right = self.parse_equality();
            left = ASTNode::new(NodeKind::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            });
        }
        left
    }

    fn parse_equality(&mut self) -> ASTNode {
        let mut left = self.parse_relational();
        while matches!(self.state.peek(), Some(&TokenType::EqualEqual) | Some(&TokenType::BangEqual)) {
            let op = self.state.advance().unwrap();
            let right = self.parse_relational();
            left = ASTNode::new(NodeKind::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            });
        }
        left
    }

    fn parse_relational(&mut self) -> ASTNode {
        let mut left = self.parse_additive();
        while matches!(self.state.peek(), Some(&TokenType::Less) | Some(&TokenType::LessEqual) | Some(&TokenType::Greater) | Some(&TokenType::GreaterEqual)) {
            let op = self.state.advance().unwrap();
            let right = self.parse_additive();
            left = ASTNode::new(NodeKind::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            });
        }
        left
    }

    fn parse_additive(&mut self) -> ASTNode {
        let mut left = self.parse_multiplicative();
        while matches!(self.state.peek(), Some(&TokenType::Plus) | Some(&TokenType::Minus)) {
            let op = self.state.advance().unwrap();
            let right = self.parse_multiplicative();
            left = ASTNode::new(NodeKind::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            });
        }
        left
    }

    fn parse_multiplicative(&mut self) -> ASTNode {
        let mut left = self.parse_unary();
        while matches!(self.state.peek(), Some(&TokenType::Star) | Some(&TokenType::Slash) | Some(&TokenType::Percent)) {
            let op = self.state.advance().unwrap();
            let right = self.parse_unary();
            left = ASTNode::new(NodeKind::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            });
        }
        left
    }

    fn parse_unary(&mut self) -> ASTNode {
        match self.state.peek() {
            Some(&TokenType::Bang) | Some(&TokenType::Minus) => {
                let op = self.state.advance().unwrap();
                let operand = self.parse_unary();
                ASTNode::new(NodeKind::UnaryOp {
                    op,
                    operand: Box::new(operand),
                })
            }
            Some(&TokenType::Ampersand) => {
                self.state.advance();
                let operand = self.parse_unary();
                ASTNode::new(NodeKind::AddressOf(Box::new(operand)))
            }
            Some(&TokenType::Star) => {
                self.state.advance();
                let operand = self.parse_unary();
                ASTNode::new(NodeKind::Deref(Box::new(operand)))
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> ASTNode {
        let mut expr = self.parse_primary();

        loop {
            match self.state.peek() {
                Some(TokenType::Dot) => {
                    self.state.advance();
                    let field = match self.state.advance() {
                        Some(TokenType::Identifier(f)) => f,
                        other => panic!("Expected field identifier after '.', got {:?}", other),
                    };
                    expr = ASTNode::new(NodeKind::FieldAccess {
                        object: Box::new(expr),
                        field,
                    });
                }
                Some(TokenType::LeftBracket) => {
                    self.state.advance();
                    let idx = self.parse_expression();
                    self.state.consume(&TokenType::RightBracket);
                    expr = ASTNode::new(NodeKind::ArrayIndex {
                        array: Box::new(expr),
                        index: Box::new(idx),
                    });
                }
                _ => break,
            }
        }
        expr
    }

    fn parse_primary(&mut self) -> ASTNode {
        match self.state.advance() {
            Some(TokenType::Number(n)) => ASTNode::new(NodeKind::Number(n)),
            Some(TokenType::IntNumber(n)) => ASTNode::new(NodeKind::IntNumber(n)),
            Some(TokenType::StringLit(s)) => ASTNode::new(NodeKind::StringLit(s)),
            Some(TokenType::Yes) => ASTNode::new(NodeKind::Boolean(true)),
            Some(TokenType::No) => ASTNode::new(NodeKind::Boolean(false)),

            Some(TokenType::Claim) => {
                self.state.consume(&TokenType::LeftParen);
                let size_expr = self.parse_expression();
                self.state.consume(&TokenType::RightParen);
                ASTNode::new(NodeKind::Claim(Box::new(size_expr)))
            }

            Some(TokenType::Purge) => {
                self.state.consume(&TokenType::LeftParen);
                let ptr_expr = self.parse_expression();
                self.state.consume(&TokenType::RightParen);
                ASTNode::new(NodeKind::Purge(Box::new(ptr_expr)))
            }

            Some(TokenType::Identifier(name)) => {
                if self.state.peek() == Some(&TokenType::LeftParen) {
                    self.state.advance();
                    let mut args = Vec::new();
                    while self.state.peek() != Some(&TokenType::RightParen) {
                        args.push(self.parse_expression());
                        if self.state.peek() == Some(&TokenType::Comma) {
                            self.state.advance();
                        }
                    }
                    self.state.consume(&TokenType::RightParen);
                    ASTNode::new(NodeKind::Call { callee: name, args })
                } else {
                    ASTNode::new(NodeKind::Identifier(name))
                }
            }

            Some(TokenType::LeftParen) => {
                let expr = self.parse_expression();
                self.state.consume(&TokenType::RightParen);
                expr
            }

            Some(TokenType::LeftBracket) => {
                let mut elements = Vec::new();
                while self.state.peek() != Some(&TokenType::RightBracket) {
                    elements.push(self.parse_expression());
                    if self.state.peek() == Some(&TokenType::Comma) {
                        self.state.advance();
                    }
                }
                self.state.consume(&TokenType::RightBracket);
                ASTNode::new(NodeKind::ArrayLiteral(elements))
            }

            other => panic!("Unexpected token in expression: {:?}", other),
        }
    }
}