use std::collections::HashMap;
use crate::core::{ASTNode, LayoutDef, NodeKind, TokenType, Type};

#[derive(Debug, Clone)]
pub enum IRInstruction {
    LoadFloatConst { dest: usize, val: f64 },
    LoadIntConst { dest: usize, val: i64 },
    LoadStringConst { dest: usize, str_id: usize },
    LoadBoolConst { dest: usize, val: bool },
    LoadVar { dest: usize, name: String },
    StoreVar { name: String, src: usize },

    Add { dest: usize, src1: usize, src2: usize, is_float: bool },
    Sub { dest: usize, src1: usize, src2: usize, is_float: bool },
    Mul { dest: usize, src1: usize, src2: usize, is_float: bool },
    Div { dest: usize, src1: usize, src2: usize, is_float: bool },
    Mod { dest: usize, src1: usize, src2: usize },

    CmpEq { dest: usize, src1: usize, src2: usize, is_float: bool },
    CmpNeq { dest: usize, src1: usize, src2: usize, is_float: bool },
    CmpLt { dest: usize, src1: usize, src2: usize, is_float: bool },
    CmpLte { dest: usize, src1: usize, src2: usize, is_float: bool },
    CmpGt { dest: usize, src1: usize, src2: usize, is_float: bool },
    CmpGte { dest: usize, src1: usize, src2: usize, is_float: bool },

    LogAnd { dest: usize, src1: usize, src2: usize },
    LogOr { dest: usize, src1: usize, src2: usize },
    LogNot { dest: usize, src: usize },

    Emit { reg: usize, reg_type: Type },

    Label(String),
    Jump(String),
    JumpIfFalse { cond_reg: usize, label: String },

    Call { dest: Option<usize>, name: String, args: Vec<usize> },
    Yield(Option<usize>),

    ComputeAddr { dest: usize, var_name: String },
    LoadDeref { dest: usize, ptr_reg: usize },
    StoreDeref { ptr_reg: usize, val_reg: usize },

    ComputeFieldAddr { dest: usize, base_reg: usize, byte_offset: usize },
    ComputeIndexAddr { dest: usize, base_reg: usize, index_reg: usize, elem_size: usize },

    Alloc { dest: usize, size_reg: usize },
    Free { ptr_reg: usize },
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct IRProc {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
    pub instructions: Vec<IRInstruction>,
}

#[derive(Debug, Clone)]
pub struct IRProgram {
    pub procs: Vec<IRProc>,
    pub main_instructions: Vec<IRInstruction>,
    pub string_literals: Vec<String>,
    pub layouts: HashMap<String, LayoutDef>,
}

impl IRProgram {
    pub fn new() -> Self {
        IRProgram {
            procs: Vec::new(),
            main_instructions: Vec::new(),
            string_literals: Vec::new(),
            layouts: HashMap::new(),
        }
    }

    pub fn add_string(&mut self, s: String) -> usize {
        if let Some(idx) = self.string_literals.iter().position(|x| x == &s) {
            idx
        } else {
            let idx = self.string_literals.len();
            self.string_literals.push(s);
            idx
        }
    }
}

pub struct IRGenerator {
    next_reg: usize,
    next_label: usize,
    symbol_types: HashMap<String, Type>,
    layouts: HashMap<String, LayoutDef>,
    in_proc: Option<String>,
}

impl IRGenerator {
    pub fn new() -> Self {
        IRGenerator {
            next_reg: 0,
            next_label: 0,
            symbol_types: HashMap::new(),
            layouts: HashMap::new(),
            in_proc: None,
        }
    }

    fn new_register(&mut self) -> usize {
        let r = self.next_reg;
        self.next_reg += 1;
        r
    }

    fn new_label(&mut self, prefix: &str) -> String {
        let l = format!("{}_{}", prefix, self.next_label);
        self.next_label += 1;
        l
    }

    pub fn generate_program(&mut self, ast: Vec<ASTNode>) -> IRProgram {
        let mut program = IRProgram::new();

        for node in &ast {
            if let NodeKind::LayoutDecl(ref def) = node.kind {
                self.layouts.insert(def.name.clone(), def.clone());
                program.layouts.insert(def.name.clone(), def.clone());
            }
        }

        let mut main_nodes = Vec::new();

        for node in ast {
            match node.kind {
                NodeKind::LayoutDecl(_) => {}
                NodeKind::ProcDecl { name, params, return_type, body } => {
                    self.generate_proc(name, params, return_type, body, &mut program);
                }
                _ => {
                    main_nodes.push(node);
                }
            }
        }

        for node in main_nodes {
            self.generate_statement(&node, &mut program, false);
        }

        program
    }

    fn emit_ins(&mut self, program: &mut IRProgram, ins: IRInstruction) {
        if let Some(ref p_name) = self.in_proc {
            if let Some(proc) = program.procs.iter_mut().find(|p| &p.name == p_name) {
                proc.instructions.push(ins);
            }
        } else {
            program.main_instructions.push(ins);
        }
    }

    fn generate_proc(
        &mut self,
        name: String,
        params: Vec<crate::core::ProcParam>,
        return_type: Type,
        body: Vec<ASTNode>,
        program: &mut IRProgram,
    ) {
        let old_in_proc = self.in_proc.clone();
        let old_symbols = self.symbol_types.clone();

        self.in_proc = Some(name.clone());

        let param_list: Vec<(String, Type)> = params
            .into_iter()
            .map(|p| {
                self.symbol_types.insert(p.name.clone(), p.param_type.clone());
                (p.name, p.param_type)
            })
            .collect();

        program.procs.push(IRProc {
            name: name.clone(),
            params: param_list,
            return_type,
            instructions: Vec::new(),
        });

        for stmt in body {
            self.generate_statement(&stmt, program, true);
        }

        self.in_proc = old_in_proc;
        self.symbol_types = old_symbols;
    }

    fn generate_statement(&mut self, node: &ASTNode, program: &mut IRProgram, inside_proc: bool) {
        match &node.kind {
            NodeKind::SlotDecl { name, slot_type, value } => {
                if let Some(val_node) = value {
                    let (val_reg, inferred_type) = self.generate_expression(val_node, program);
                    let actual_type = slot_type.clone().unwrap_or(inferred_type);
                    self.symbol_types.insert(name.clone(), actual_type);
                    if let Some(vr) = val_reg {
                        self.emit_ins(program, IRInstruction::StoreVar {
                            name: name.clone(),
                            src: vr,
                        });
                    }
                } else if let Some(st) = slot_type {
                    self.symbol_types.insert(name.clone(), st.clone());
                }
            }

            NodeKind::Assignment { target, value } => {
                let (val_reg, _) = self.generate_expression(value, program);
                let val_r = val_reg.expect("Right hand side of assignment produced no value");

                match &target.kind {
                    NodeKind::Identifier(name) => {
                        self.emit_ins(program, IRInstruction::StoreVar {
                            name: name.clone(),
                            src: val_r,
                        });
                    }
                    NodeKind::Deref(inner) => {
                        let (ptr_reg, _) = self.generate_expression(inner, program);
                        self.emit_ins(program, IRInstruction::StoreDeref {
                            ptr_reg: ptr_reg.unwrap(),
                            val_reg: val_r,
                        });
                    }
                    NodeKind::FieldAccess { .. } => {
                        let (addr_reg, _) = self.generate_address(target, program);
                        self.emit_ins(program, IRInstruction::StoreDeref {
                            ptr_reg: addr_reg,
                            val_reg: val_r,
                        });
                    }
                    NodeKind::ArrayIndex { .. } => {
                        let (addr_reg, _) = self.generate_address(target, program);
                        self.emit_ins(program, IRInstruction::StoreDeref {
                            ptr_reg: addr_reg,
                            val_reg: val_r,
                        });
                    }
                    _ => panic!("Invalid assignment target"),
                }
            }

            NodeKind::Emit(expr) => {
                let (reg, expr_type) = self.generate_expression(expr, program);
                if let Some(r) = reg {
                    self.emit_ins(program, IRInstruction::Emit { reg: r, reg_type: expr_type });
                }
            }

            NodeKind::When { condition, then_branch, else_branch } => {
                let (cond_reg, _) = self.generate_expression(condition, program);
                let c_reg = cond_reg.expect("Condition must return boolean value");
                let else_label = self.new_label("else");
                let end_label = self.new_label("end_when");

                self.emit_ins(program, IRInstruction::JumpIfFalse {
                    cond_reg: c_reg,
                    label: else_label.clone(),
                });

                for stmt in then_branch {
                    self.generate_statement(stmt, program, inside_proc);
                }

                if else_branch.is_some() {
                    self.emit_ins(program, IRInstruction::Jump(end_label.clone()));
                }

                self.emit_ins(program, IRInstruction::Label(else_label));

                if let Some(eb) = else_branch {
                    for stmt in eb {
                        self.generate_statement(stmt, program, inside_proc);
                    }
                    self.emit_ins(program, IRInstruction::Label(end_label));
                }
            }

            NodeKind::During { condition, body } => {
                let start_label = self.new_label("during_start");
                let end_label = self.new_label("during_end");

                self.emit_ins(program, IRInstruction::Label(start_label.clone()));

                let (cond_reg, _) = self.generate_expression(condition, program);
                self.emit_ins(program, IRInstruction::JumpIfFalse {
                    cond_reg: cond_reg.unwrap(),
                    label: end_label.clone(),
                });

                for stmt in body {
                    self.generate_statement(stmt, program, inside_proc);
                }

                self.emit_ins(program, IRInstruction::Jump(start_label));
                self.emit_ins(program, IRInstruction::Label(end_label));
            }

            NodeKind::Yield(expr) => {
                let r = if let Some(e) = expr {
                    let (reg, _) = self.generate_expression(e, program);
                    reg
                } else {
                    None
                };
                self.emit_ins(program, IRInstruction::Yield(r));
            }

            NodeKind::Block(stmts) => {
                for s in stmts {
                    self.generate_statement(s, program, inside_proc);
                }
            }

            NodeKind::ExprStatement(expr) => {
                self.generate_expression(expr, program);
            }

            _ => {}
        }
    }

    fn generate_expression(&mut self, node: &ASTNode, program: &mut IRProgram) -> (Option<usize>, Type) {
        match &node.kind {
            NodeKind::Number(n) => {
                let dest = self.new_register();
                self.emit_ins(program, IRInstruction::LoadFloatConst { dest, val: *n });
                (Some(dest), Type::Num)
            }
            NodeKind::IntNumber(n) => {
                let dest = self.new_register();
                self.emit_ins(program, IRInstruction::LoadIntConst { dest, val: *n });
                (Some(dest), Type::Int)
            }
            NodeKind::StringLit(s) => {
                let str_id = program.add_string(s.clone());
                let dest = self.new_register();
                self.emit_ins(program, IRInstruction::LoadStringConst { dest, str_id });
                (Some(dest), Type::Text)
            }
            NodeKind::Boolean(b) => {
                let dest = self.new_register();
                self.emit_ins(program, IRInstruction::LoadBoolConst { dest, val: *b });
                (Some(dest), Type::Flag)
            }
            NodeKind::Identifier(name) => {
                let dest = self.new_register();
                let sym_type = self.symbol_types.get(name).cloned().unwrap_or(Type::Int);
                self.emit_ins(program, IRInstruction::LoadVar { dest, name: name.clone() });
                (Some(dest), sym_type)
            }
            NodeKind::BinaryOp { left, op, right } => {
                let (l_reg, l_type) = self.generate_expression(left, program);
                let (r_reg, _) = self.generate_expression(right, program);
                let dest = self.new_register();
                let is_float = l_type == Type::Num;

                match op {
                    TokenType::Plus => self.emit_ins(program, IRInstruction::Add { dest, src1: l_reg.unwrap(), src2: r_reg.unwrap(), is_float }),
                    TokenType::Minus => self.emit_ins(program, IRInstruction::Sub { dest, src1: l_reg.unwrap(), src2: r_reg.unwrap(), is_float }),
                    TokenType::Star => self.emit_ins(program, IRInstruction::Mul { dest, src1: l_reg.unwrap(), src2: r_reg.unwrap(), is_float }),
                    TokenType::Slash => self.emit_ins(program, IRInstruction::Div { dest, src1: l_reg.unwrap(), src2: r_reg.unwrap(), is_float }),
                    TokenType::Percent => self.emit_ins(program, IRInstruction::Mod { dest, src1: l_reg.unwrap(), src2: r_reg.unwrap() }),

                    TokenType::EqualEqual => {
                        self.emit_ins(program, IRInstruction::CmpEq { dest, src1: l_reg.unwrap(), src2: r_reg.unwrap(), is_float });
                        return (Some(dest), Type::Flag);
                    }
                    TokenType::BangEqual => {
                        self.emit_ins(program, IRInstruction::CmpNeq { dest, src1: l_reg.unwrap(), src2: r_reg.unwrap(), is_float });
                        return (Some(dest), Type::Flag);
                    }
                    TokenType::Less => {
                        self.emit_ins(program, IRInstruction::CmpLt { dest, src1: l_reg.unwrap(), src2: r_reg.unwrap(), is_float });
                        return (Some(dest), Type::Flag);
                    }
                    TokenType::LessEqual => {
                        self.emit_ins(program, IRInstruction::CmpLte { dest, src1: l_reg.unwrap(), src2: r_reg.unwrap(), is_float });
                        return (Some(dest), Type::Flag);
                    }
                    TokenType::Greater => {
                        self.emit_ins(program, IRInstruction::CmpGt { dest, src1: l_reg.unwrap(), src2: r_reg.unwrap(), is_float });
                        return (Some(dest), Type::Flag);
                    }
                    TokenType::GreaterEqual => {
                        self.emit_ins(program, IRInstruction::CmpGte { dest, src1: l_reg.unwrap(), src2: r_reg.unwrap(), is_float });
                        return (Some(dest), Type::Flag);
                    }
                    TokenType::AmpAmp => {
                        self.emit_ins(program, IRInstruction::LogAnd { dest, src1: l_reg.unwrap(), src2: r_reg.unwrap() });
                        return (Some(dest), Type::Flag);
                    }
                    TokenType::PipePipe => {
                        self.emit_ins(program, IRInstruction::LogOr { dest, src1: l_reg.unwrap(), src2: r_reg.unwrap() });
                        return (Some(dest), Type::Flag);
                    }
                    _ => panic!("Unsupported binary operator {:?}", op),
                }
                (Some(dest), l_type)
            }
            NodeKind::UnaryOp { op, operand } => {
                let (reg, op_type) = self.generate_expression(operand, program);
                let dest = self.new_register();
                match op {
                    TokenType::Bang => {
                        self.emit_ins(program, IRInstruction::LogNot { dest, src: reg.unwrap() });
                        (Some(dest), Type::Flag)
                    }
                    TokenType::Minus => {
                        let zero = self.new_register();
                        if op_type == Type::Num {
                            self.emit_ins(program, IRInstruction::LoadFloatConst { dest: zero, val: 0.0 });
                            self.emit_ins(program, IRInstruction::Sub { dest, src1: zero, src2: reg.unwrap(), is_float: true });
                        } else {
                            self.emit_ins(program, IRInstruction::LoadIntConst { dest: zero, val: 0 });
                            self.emit_ins(program, IRInstruction::Sub { dest, src1: zero, src2: reg.unwrap(), is_float: false });
                        }
                        (Some(dest), op_type)
                    }
                    _ => panic!("Unsupported unary operator"),
                }
            }
            NodeKind::Call { callee, args } => {
                let mut arg_regs = Vec::new();
                for a in args {
                    let (ar, _) = self.generate_expression(a, program);
                    arg_regs.push(ar.unwrap());
                }
                let dest = self.new_register();
                self.emit_ins(program, IRInstruction::Call {
                    dest: Some(dest),
                    name: callee.clone(),
                    args: arg_regs,
                });
                (Some(dest), Type::Int)
            }
            NodeKind::AddressOf(inner) => {
                let (addr_reg, inner_type) = self.generate_address(inner, program);
                (Some(addr_reg), Type::Pointer(Box::new(inner_type)))
            }
            NodeKind::Deref(inner) => {
                let (ptr_reg, ptr_type) = self.generate_expression(inner, program);
                let elem_type = match ptr_type {
                    Type::Pointer(t) => *t,
                    _ => Type::Int,
                };
                let dest = self.new_register();
                self.emit_ins(program, IRInstruction::LoadDeref { dest, ptr_reg: ptr_reg.unwrap() });
                (Some(dest), elem_type)
            }
            NodeKind::Claim(size_expr) => {
                let (s_reg, _) = self.generate_expression(size_expr, program);
                let dest = self.new_register();
                self.emit_ins(program, IRInstruction::Alloc { dest, size_reg: s_reg.unwrap() });
                (Some(dest), Type::Pointer(Box::new(Type::None)))
            }
            NodeKind::Purge(ptr_expr) => {
                let (p_reg, _) = self.generate_expression(ptr_expr, program);
                self.emit_ins(program, IRInstruction::Free { ptr_reg: p_reg.unwrap() });
                (None, Type::None)
            }
            NodeKind::ArrayIndex { .. } => {
                let (addr_reg, elem_type) = self.generate_address(node, program);
                let dest = self.new_register();
                self.emit_ins(program, IRInstruction::LoadDeref { dest, ptr_reg: addr_reg });
                (Some(dest), elem_type)
            }
            NodeKind::FieldAccess { .. } => {
                let (addr_reg, field_type) = self.generate_address(node, program);
                let dest = self.new_register();
                self.emit_ins(program, IRInstruction::LoadDeref { dest, ptr_reg: addr_reg });
                (Some(dest), field_type)
            }
            _ => (None, Type::None),
        }
    }

    fn generate_address(&mut self, node: &ASTNode, program: &mut IRProgram) -> (usize, Type) {
        match &node.kind {
            NodeKind::Identifier(name) => {
                let dest = self.new_register();
                let sym_type = self.symbol_types.get(name).cloned().unwrap_or(Type::Int);
                self.emit_ins(program, IRInstruction::ComputeAddr { dest, var_name: name.clone() });
                (dest, sym_type)
            }
            NodeKind::Deref(inner) => {
                let (ptr_reg, ptr_type) = self.generate_expression(inner, program);
                let target_type = match ptr_type {
                    Type::Pointer(t) => *t,
                    _ => Type::Int,
                };
                (ptr_reg.unwrap(), target_type)
            }
            NodeKind::FieldAccess { object, field } => {
                let (base_addr, obj_type) = self.generate_address(object, program);
                let layout_name = match obj_type {
                    Type::Layout(n) => n,
                    Type::Pointer(inner) => match *inner {
                        Type::Layout(n) => n,
                        _ => panic!("Expected layout pointer"),
                    },
                    _ => panic!("Expected layout type for field access"),
                };

                let (byte_offset, field_type) = {
                    let def = self.layouts.get(&layout_name).unwrap_or_else(|| panic!("Unknown layout {}", layout_name));
                    let field_info = def.fields.iter().find(|f| &f.name == field).unwrap_or_else(|| panic!("Unknown field {}", field));
                    (field_info.offset, field_info.field_type.clone())
                };

                let dest = self.new_register();
                self.emit_ins(program, IRInstruction::ComputeFieldAddr {
                    dest,
                    base_reg: base_addr,
                    byte_offset,
                });
                (dest, field_type)
            }
            NodeKind::ArrayIndex { array, index } => {
                let (base_addr, arr_type) = self.generate_address(array, program);
                let elem_type = match arr_type {
                    Type::Array(t, _) => *t,
                    Type::Pointer(t) => *t,
                    _ => Type::Int,
                };
                let (idx_reg, _) = self.generate_expression(index, program);
                let dest = self.new_register();
                self.emit_ins(program, IRInstruction::ComputeIndexAddr {
                    dest,
                    base_reg: base_addr,
                    index_reg: idx_reg.unwrap(),
                    elem_size: 8,
                });
                (dest, elem_type)
            }
            _ => panic!("Cannot take address of this node"),
        }
    }
}