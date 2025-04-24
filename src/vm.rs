// src/vm.rs

use crate::error::XplError;
use crate::parser::{BinOp, Expr, Program, Stmt};
use std::collections::HashMap;

pub struct VM {
    vars: HashMap<String, i64>,
}

impl VM {
    pub fn new() -> Self {
        VM {
            vars: HashMap::new(),
        }
    }

    /// Execute a Program and return printed outputs
    pub fn run(&mut self, prog: &Program) -> Result<Vec<String>, XplError> {
        let mut outputs = Vec::new();
        // Find main function
        let main_fn = prog
            .functions
            .get("main")
            .ok_or_else(|| XplError::Semantic("No main function".to_string()))?;
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
                        Expr::VarRef(name) => {
                            self.vars.get(name).map_or(name.clone(), |v| v.to_string())
                        }
                        Expr::Call(_, _) | Expr::BinaryOp(..) => {
                            self.eval_expr(expr, prog)?.to_string()
                        }
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
                                    Expr::VarRef(name) => {
                                        self.vars.get(name).map_or(name.clone(), |v| v.to_string())
                                    }
                                    Expr::Call(_, _) | Expr::BinaryOp(..) => {
                                        self.eval_expr(expr, prog)?.to_string()
                                    }
                                };
                                outputs.push(out);
                            }
                            _ => {}
                        }
                    }
                }
                Stmt::Return(_) => {
                    // ignore return in main
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
                            return Err(XplError::Semantic("Division by zero".to_string()));
                        }
                        left / right
                    }
                    BinOp::Modulus => {
                        if right == 0 {
                            return Err(XplError::Semantic("Division by zero".to_string()));
                        }
                        left % right
                    }
                };
                return Ok(res);
            }
            Expr::LiteralInt(i) => Ok(*i),
            Expr::VarRef(name) => self
                .vars
                .get(name)
                .cloned()
                .ok_or_else(|| XplError::Semantic(format!("Undefined variable {}", name))),
            Expr::Call(name, args) => {
                // Evaluate argument expressions
                let mut arg_vals = Vec::new();
                for a in args {
                    arg_vals.push(self.eval_expr(a, prog)?);
                }
                // Call user-defined function
                self.call_function(prog, name, arg_vals)
            }
            _ => Err(XplError::Semantic(
                "Unsupported expression in eval".to_string(),
            )),
        }
    }

    /// Call a user-defined function and return its integer return value
    fn call_function(
        &mut self,
        prog: &Program,
        name: &str,
        args: Vec<i64>,
    ) -> Result<i64, XplError> {
        let func = prog
            .functions
            .get(name)
            .ok_or_else(|| XplError::Semantic(format!("Undefined function {}", name)))?;
        if func.params.len() != args.len() {
            return Err(XplError::Semantic(format!(
                "Expected {} args for function '{}', got {}",
                func.params.len(),
                name,
                args.len()
            )));
        }
        // Setup local frame
        let mut locals = std::collections::HashMap::new();
        for (p, v) in func.params.iter().zip(args) {
            locals.insert(p.clone(), v);
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
