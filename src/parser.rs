// src/parser.rs

use crate::error::XplError;
use std::collections::HashMap;
use std::fs::File;
use xmltree::{Element, XMLNode};

#[derive(Debug, Clone)]
pub struct Program {
    pub description: Option<String>,
    pub functions: HashMap<String, Function>,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ptype: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub description: Option<String>,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Assign {
        var: String,
        expr: Expr,
    },
    Print(Expr),
    If {
        cond: Expr,
        then_body: Vec<Stmt>,
        else_body: Vec<Stmt>,
    },
    Return(Expr),
    Call(String, Vec<Expr>),
    Loop {
        count: Expr,
        body: Vec<Stmt>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    LiteralInt(i64),
    LiteralStr(String),
    VarRef(String),
    Call(String, Vec<Expr>),
    BinaryOp(BinOp, Box<Expr>, Box<Expr>),
}

/// Parse an XPL file into a Program AST
pub fn parse_file(path: &str) -> Result<Program, XplError> {
    let file = File::open(path).map_err(|e| XplError::Io {
        source: e,
        file: path.to_string(),
    })?;
    let root = Element::parse(file).map_err(|e| XplError::Xml {
        source: e,
        file: path.to_string(),
    })?;
    let mut functions = HashMap::new();
    // optional program-level description
    let prog_desc = root
        .get_child("description")
        .and_then(|d| Some(d.get_text().unwrap_or_default().trim().to_string()));
    // Process include only for program roots (to load libs)
    if root.name == "program" {
        if let Some(include_list) = root.attributes.get("include") {
            let script_dir = std::path::Path::new(path)
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            for inc in include_list.split(',').map(|s| s.trim()) {
                // try script-relative first, then workspace-relative
                let rel_path = script_dir.join(inc);
                let inc_path = if rel_path.exists() {
                    rel_path
                } else {
                    std::path::Path::new(inc).to_path_buf()
                };
                let included = parse_file(inc_path.to_str().unwrap())?;
                functions.extend(included.functions);
            }
        }
    }
    for node in &root.children {
        if let XMLNode::Element(elem) = node {
            if elem.name == "function" {
                // optional function-level description
                let func_desc = elem
                    .get_child("description")
                    .and_then(|d| Some(d.get_text().unwrap_or_default().trim().to_string()));
                let name = elem.attributes.get("name").cloned().unwrap_or_default();
                // collect parameters with optional type and description
                let mut params = Vec::new();
                for c in &elem.children {
                    if let XMLNode::Element(e) = c {
                        if e.name == "param" {
                            let name = e.attributes.get("name").cloned().unwrap_or_default();
                            let ptype = e.attributes.get("type").cloned();
                            // optional description child
                            let desc = e.get_child("description").and_then(|d| {
                                Some(d.get_text().unwrap_or_default().trim().to_string())
                            });
                            params.push(Param {
                                name,
                                ptype,
                                description: desc,
                            });
                        }
                    }
                }
                let mut body = Vec::new();
                if let Some(body_elem) = elem.get_child("body") {
                    for stmt_node in &body_elem.children {
                        if let XMLNode::Element(stmt_elem) = stmt_node {
                            match stmt_elem.name.as_str() {
                                "loop" => {
                                    // parse loop count
                                    let times_str = stmt_elem
                                        .attributes
                                        .get("times")
                                        .cloned()
                                        .unwrap_or_else(|| "0".into());
                                    let count_expr = parse_text_expr(&times_str);
                                    // parse loop body statements
                                    let mut loop_body = Vec::new();
                                    for loop_node in &stmt_elem.children {
                                        if let XMLNode::Element(e) = loop_node {
                                            match e.name.as_str() {
                                                "print" => {
                                                    let txt = e
                                                        .get_text()
                                                        .unwrap_or_default()
                                                        .trim()
                                                        .to_string();
                                                    let expr = if txt.starts_with('"')
                                                        && txt.ends_with('"')
                                                    {
                                                        Expr::LiteralStr(
                                                            txt.trim_matches('"').to_string(),
                                                        )
                                                    } else if let Ok(i) = txt.parse::<i64>() {
                                                        Expr::LiteralInt(i)
                                                    } else {
                                                        Expr::VarRef(txt)
                                                    };
                                                    loop_body.push(Stmt::Print(expr));
                                                }
                                                "assign" => {
                                                    let var = e
                                                        .attributes
                                                        .get("var")
                                                        .cloned()
                                                        .unwrap_or_default();
                                                    if let Some(XMLNode::Element(expr_elem)) =
                                                        e.children.get(0)
                                                    {
                                                        let expr = parse_expr(expr_elem)?;
                                                        loop_body.push(Stmt::Assign { var, expr });
                                                    }
                                                }
                                                "call" => {
                                                    let expr = parse_expr(e)?;
                                                    if let Expr::Call(name, args) = expr {
                                                        loop_body.push(Stmt::Call(name, args));
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                    body.push(Stmt::Loop {
                                        count: count_expr,
                                        body: loop_body,
                                    });
                                }
                                "call" => {
                                    // standalone call statement
                                    let expr = parse_expr(stmt_elem)?;
                                    if let Expr::Call(name, args) = expr {
                                        body.push(Stmt::Call(name, args));
                                    }
                                }
                                "return" => {
                                    let expr = if let Some(XMLNode::Element(e)) =
                                        stmt_elem.children.get(0)
                                    {
                                        parse_expr(e)?
                                    } else {
                                        let txt = stmt_elem.get_text().unwrap_or_default();
                                        parse_text_expr(&txt)
                                    };
                                    body.push(Stmt::Return(expr));
                                }
                                "if" => {
                                    let cond_elem =
                                        stmt_elem.get_child("condition").ok_or_else(|| {
                                            XplError::Semantic {
                                                msg: "Missing condition".to_string(),
                                                file: path.to_string(),
                                                line: 0,
                                                col: 0,
                                            }
                                        })?;
                                    // either inner element or text
                                    let cond_expr = if let Some(XMLNode::Element(e)) =
                                        cond_elem.children.get(0)
                                    {
                                        parse_expr(e)?
                                    } else {
                                        let txt = cond_elem
                                            .get_text()
                                            .unwrap_or_default()
                                            .trim()
                                            .to_string();
                                        if let Ok(i) = txt.parse::<i64>() {
                                            Expr::LiteralInt(i)
                                        } else {
                                            Expr::VarRef(txt)
                                        }
                                    };
                                    // then block
                                    let then_elem =
                                        stmt_elem.get_child("then").ok_or_else(|| {
                                            XplError::Semantic {
                                                msg: "Missing then block".to_string(),
                                                file: path.to_string(),
                                                line: 0,
                                                col: 0,
                                            }
                                        })?;
                                    let mut then_body = Vec::new();
                                    for then_node in &then_elem.children {
                                        if let XMLNode::Element(e) = then_node {
                                            if e.name == "print" {
                                                let txt = e
                                                    .get_text()
                                                    .unwrap_or_default()
                                                    .trim()
                                                    .to_string();
                                                let expr =
                                                    if txt.starts_with('"') && txt.ends_with('"') {
                                                        Expr::LiteralStr(
                                                            txt.trim_matches('"').to_string(),
                                                        )
                                                    } else if let Ok(i) = txt.parse::<i64>() {
                                                        Expr::LiteralInt(i)
                                                    } else {
                                                        Expr::VarRef(txt)
                                                    };
                                                then_body.push(Stmt::Print(expr));
                                            }
                                        }
                                    }
                                    // else block
                                    let else_elem =
                                        stmt_elem.get_child("else").ok_or_else(|| {
                                            XplError::Semantic {
                                                msg: "Missing else block".to_string(),
                                                file: path.to_string(),
                                                line: 0,
                                                col: 0,
                                            }
                                        })?;
                                    let mut else_body = Vec::new();
                                    for else_node in &else_elem.children {
                                        if let XMLNode::Element(e) = else_node {
                                            if e.name == "print" {
                                                let txt = e
                                                    .get_text()
                                                    .unwrap_or_default()
                                                    .trim()
                                                    .to_string();
                                                let expr =
                                                    if txt.starts_with('"') && txt.ends_with('"') {
                                                        Expr::LiteralStr(
                                                            txt.trim_matches('"').to_string(),
                                                        )
                                                    } else if let Ok(i) = txt.parse::<i64>() {
                                                        Expr::LiteralInt(i)
                                                    } else {
                                                        Expr::VarRef(txt)
                                                    };
                                                else_body.push(Stmt::Print(expr));
                                            }
                                        }
                                    }
                                    body.push(Stmt::If {
                                        cond: cond_expr,
                                        then_body,
                                        else_body,
                                    });
                                }
                                "assign" => {
                                    let var = stmt_elem
                                        .attributes
                                        .get("var")
                                        .cloned()
                                        .unwrap_or_default();
                                    if let Some(XMLNode::Element(expr_elem)) =
                                        stmt_elem.children.get(0)
                                    {
                                        let expr = parse_expr(expr_elem)?;
                                        body.push(Stmt::Assign { var, expr });
                                    }
                                }
                                "print" => {
                                    let txt =
                                        stmt_elem.get_text().unwrap_or_default().trim().to_string();
                                    let expr = if txt.starts_with('"') && txt.ends_with('"') {
                                        Expr::LiteralStr(txt.trim_matches('"').to_string())
                                    } else if let Ok(i) = txt.parse::<i64>() {
                                        Expr::LiteralInt(i)
                                    } else {
                                        Expr::VarRef(txt)
                                    };
                                    body.push(Stmt::Print(expr));
                                }
                                _ => {}
                            }
                        }
                    }
                }
                functions.insert(
                    name.clone(),
                    Function {
                        name,
                        description: func_desc,
                        params,
                        body,
                    },
                );
            }
        }
    }
    Ok(Program {
        description: prog_desc,
        functions,
    })
}

/// Parse a simple text expression, supporting infix ops
fn parse_text_expr(txt: &str) -> Expr {
    let t = txt.trim();
    // infix pattern
    let parts: Vec<&str> = t.split_whitespace().collect();
    if parts.len() == 3 {
        let left = parse_text_expr(parts[0]);
        let right = parse_text_expr(parts[2]);
        let op = match parts[1] {
            "+" => BinOp::Add,
            "-" => BinOp::Subtract,
            "*" => BinOp::Multiply,
            "/" => BinOp::Divide,
            "%" => BinOp::Modulus,
            _ => return Expr::VarRef(t.to_string()),
        };
        return Expr::BinaryOp(op, Box::new(left), Box::new(right));
    }
    if t.starts_with('"') && t.ends_with('"') {
        Expr::LiteralStr(t.trim_matches('"').to_string())
    } else if let Ok(i) = t.parse::<i64>() {
        Expr::LiteralInt(i)
    } else {
        Expr::VarRef(t.to_string())
    }
}

fn parse_expr(elem: &Element) -> Result<Expr, XplError> {
    // Only handle explicit <call> elements
    if elem.name == "call" {
        let func = elem.attributes.get("function").cloned().unwrap_or_default();
        let args = elem
            .children
            .iter()
            .filter_map(|node| {
                if let XMLNode::Element(e) = node {
                    if e.name == "param" { Some(e) } else { None }
                } else {
                    None
                }
            })
            .map(|p| {
                if let Some(child) = p.children.iter().find_map(|n| {
                    if let XMLNode::Element(e) = n {
                        Some(e)
                    } else {
                        None
                    }
                }) {
                    parse_expr(child)
                } else {
                    let txt = p.get_text().unwrap_or_default();
                    Ok(parse_text_expr(&txt))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(Expr::Call(func, args));
    }
    // Otherwise literal or varref
    let txt = elem.get_text().unwrap_or_default().trim().to_string();
    if txt.starts_with('"') && txt.ends_with('"') {
        Ok(Expr::LiteralStr(txt.trim_matches('"').to_string()))
    } else if let Ok(i) = txt.parse::<i64>() {
        Ok(Expr::LiteralInt(i))
    } else {
        Ok(Expr::VarRef(txt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_simple_print() {
        let tmp = "<program name=\"t\" version=\"1.0\"><function name=\"main\"><body><print>10</print></body></function></program>";
        let path = std::env::temp_dir().join("simple.xpl");
        std::fs::write(&path, tmp).unwrap();
        let prog = parse_file(path.to_str().unwrap()).unwrap();
        let func = prog.functions.get("main").unwrap();
        assert_eq!(func.body, vec![Stmt::Print(Expr::LiteralInt(10))]);
    }
}
