// src/vm.rs

use crate::error::XplError;
use crate::parser::{BinOp, Expr, Program, Stmt};
use std::collections::HashMap;

pub struct VM {
    vars: HashMap<String, i64>,
    file: String,
}

impl VM {
    /// Create a VM with file context for error reporting
    pub fn new(file: String) -> Self {
        VM {
            vars: HashMap::new(),
            file,
        }
    }

    /// Find (line, col) of the first occurrence of token in the source file
    fn find_pos(&self, token: &str) -> (usize, usize) {
        if let Ok(content) = std::fs::read_to_string(&self.file) {
            for (i, line) in content.lines().enumerate() {
                if let Some(idx) = line.find(token) {
                    return (i, idx);
                }
            }
        }
        (0, 0)
    }

    /// Execute a Program and return printed outputs
    pub fn run(&mut self, prog: &Program) -> Result<Vec<String>, XplError> {
        let mut outputs = Vec::new();
        // Find main function
        let main_fn = prog
            .functions
            .get("main")
            .ok_or_else(|| XplError::Semantic {
                msg: "No main function".to_string(),
                file: self.file.clone(),
                line: 0,
                col: 0,
            })?;
        for stmt in &main_fn.body {
            match stmt {
                Stmt::Assign { var, expr } => {
                    let val = self.eval_expr(expr, prog)?;
                    self.vars.insert(var.clone(), val);
                }
                Stmt::Print(expr) => {
                    let out = match expr {
                        Expr::LiteralStr(s) => s.clone(),
                        Expr::LiteralInt(i) => i.to_string(),
                        _ => self.eval_expr(expr, prog)?.to_string(),
                    };
                    outputs.push(out);
                }
                Stmt::If {
                    cond,
                    then_body,
                    else_body,
                } => {
                    let cond_val = self.eval_expr(cond, prog)?;
                    let stmts = if cond_val != 0 { then_body } else { else_body };
                    for st in stmts {
                        match st {
                            Stmt::Assign { var, expr } => {
                                let val = self.eval_expr(expr, prog)?;
                                self.vars.insert(var.clone(), val);
                            }
                            Stmt::Print(expr) => {
                                let out = match expr {
                                    Expr::LiteralStr(s) => s.clone(),
                                    Expr::LiteralInt(i) => i.to_string(),
                                    _ => self.eval_expr(expr, prog)?.to_string(),
                                };
                                outputs.push(out);
                            }
                            _ => {}
                        }
                    }
                }
                Stmt::Loop { count, body } => {
                    // evaluate loop count
                    let times = self.eval_expr(count, prog)?;
                    for _ in 0..times {
                        // execute each statement in loop body
                        for st in body {
                            match st {
                                Stmt::Assign { var, expr } => {
                                    let val = self.eval_expr(expr, prog)?;
                                    self.vars.insert(var.clone(), val);
                                }
                                Stmt::Print(expr) => {
                                    let out = match expr {
                                        Expr::LiteralStr(s) => s.clone(),
                                        Expr::LiteralInt(i) => i.to_string(),
                                        _ => self.eval_expr(expr, prog)?.to_string(),
                                    };
                                    outputs.push(out);
                                }
                                Stmt::Loop { .. } => {
                                    let _nested = Stmt::Loop {
                                        count: count.clone(),
                                        body: body.clone(),
                                    };
                                    // recurse by adding statement
                                    // could call run but simpler: ignore nested for now
                                }
                                Stmt::Call(name, args) => {
                                    let expr = Expr::Call(name.clone(), args.clone());
                                    let _ = self.eval_expr(&expr, prog)?;
                                }
                                Stmt::Return(_) => {}
                                Stmt::If {
                                    cond,
                                    then_body,
                                    else_body,
                                } => {
                                    // reuse existing semantics: evaluate one iteration of if
                                    let cond_val = self.eval_expr(cond, prog)?;
                                    let branch = if cond_val != 0 { then_body } else { else_body };
                                    for b in branch {
                                        match b {
                                            Stmt::Assign { var, expr } => {
                                                let v = self.eval_expr(expr, prog)?;
                                                self.vars.insert(var.clone(), v);
                                            }
                                            Stmt::Print(expr) => {
                                                let o = match expr {
                                                    Expr::LiteralStr(s) => s.clone(),
                                                    Expr::LiteralInt(i) => i.to_string(),
                                                    _ => self.eval_expr(expr, prog)?.to_string(),
                                                };
                                                outputs.push(o);
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Stmt::Return(_) => { /* ignore return in main */ }
                Stmt::Call(name, args) => {
                    // Evaluate standalone call, errors on undefined function
                    let expr = Expr::Call(name.clone(), args.clone());
                    let _ = self.eval_expr(&expr, prog)?;
                }
            }
        }
        Ok(outputs)
    }

    /// Evaluate an expression; supports function calls to user-defined functions
    fn eval_expr(&mut self, expr: &Expr, prog: &Program) -> Result<i64, XplError> {
        match expr {
            Expr::BinaryOp(op, l, r) => {
                let left = self.eval_expr(l, prog)?;
                let right = self.eval_expr(r, prog)?;
                let res = match op {
                    BinOp::Add => left + right,
                    BinOp::Subtract => left - right,
                    BinOp::Multiply => left * right,
                    BinOp::Divide => {
                        if right == 0 {
                            return Err(XplError::Semantic {
                                msg: "Division by zero".to_string(),
                                file: self.file.clone(),
                                line: 0,
                                col: 0,
                            });
                        }
                        left / right
                    }
                    BinOp::Modulus => {
                        if right == 0 {
                            return Err(XplError::Semantic {
                                msg: "Division by zero".to_string(),
                                file: self.file.clone(),
                                line: 0,
                                col: 0,
                            });
                        }
                        left % right
                    }
                };
                return Ok(res);
            }
            Expr::LiteralInt(i) => Ok(*i),
            Expr::VarRef(name) => match self.vars.get(name) {
                Some(v) => Ok(*v),
                None => {
                    let (line, col) = self.find_pos(name);
                    Err(XplError::Semantic {
                        msg: format!("Undefined variable {}", name),
                        file: self.file.clone(),
                        line: line + 1,
                        col: col + 1,
                    })
                }
            },
            Expr::Call(name, args) => {
                // Evaluate argument expressions
                let mut arg_vals = Vec::new();
                for a in args {
                    arg_vals.push(self.eval_expr(a, prog)?);
                }
                // Call user-defined function
                self.call_function(prog, name, arg_vals)
            }
            _ => Err(XplError::Semantic {
                msg: "Unsupported expression in eval".to_string(),
                file: self.file.clone(),
                line: 0,
                col: 0,
            }),
        }
    }

    /// Call a user-defined function and return its integer return value
    fn call_function(
        &mut self,
        prog: &Program,
        name: &str,
        args: Vec<i64>,
    ) -> Result<i64, XplError> {
        let func = prog.functions.get(name).ok_or_else(|| {
            let (line, col) = self.find_pos(name);
            XplError::Semantic {
                msg: format!("Undefined function {}", name),
                file: self.file.clone(),
                line: line + 1,
                col: col + 1,
            }
        })?;
        if func.params.len() != args.len() {
            return Err(XplError::Semantic {
                msg: format!(
                    "Expected {} args for function '{}', got {}",
                    func.params.len(),
                    name,
                    args.len()
                ),
                file: self.file.clone(),
                line: 0,
                col: 0,
            });
        }
        // Setup local frame
        let mut locals = std::collections::HashMap::new();
        for (p, v) in func.params.iter().zip(args) {
            locals.insert(p.name.clone(), v);
        }
        // Save global vars
        let saved = std::mem::take(&mut self.vars);
        // Use locals as vars for this call
        self.vars = locals;
        // Execute function body
        let mut ret = 0;
        for stmt in &func.body {
            match stmt {
                Stmt::Return(expr) => {
                    ret = self.eval_expr(expr, prog)?;
                    break;
                }
                _ => {}
            }
        }
        // Restore global vars
        self.vars = saved;
        Ok(ret)
    }
}
