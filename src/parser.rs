use super::scanner;
use super::reader;

#[derive(Debug, PartialEq)]
pub struct Atom {
    pub pars : Vec<String>,
    pub src  : Option<String>,
    pub dest : Option<String>
}

#[derive(Debug, PartialEq)]
pub enum AST {
    Op(Atom),                 // exec < src > dest
    And(Box<AST>, Box<AST>),  // a & b
    Pipe(Atom, Box<AST>),     // a | b
    Error,
}

fn match_expr(r : &mut reader::Reader) -> (scanner::Token, AST) {
    let mut next = scanner::next_token(r);
    let mut v = Vec::new();
    match next {
        scanner::Token::Name(name) => {
            v.push(name);
        },
        _ => panic!("Name(str) required, {:?} found", next),
    }

    next = scanner::next_token(r);
    while match next {
        scanner::Token::Name(name2) => {
            v.push(name2);
            next = scanner::next_token(r);
            true
        },
        _ => false,
    } {}

    let src = match next {
        scanner::Token::Opr(c) => {
            if c == '<' {
                next = scanner::next_token(r);
                match next {
                    scanner::Token::Name(from) => {
                        next = scanner::next_token(r);
                        Some(from)
                    }
                    _ => panic!("Name expected, {:?} found", next),
                }
            } else {
                None
            }
        }
        _ => {
            None
        }
    };

    let dest = match next {
        scanner::Token::Opr(c) => {
            if c == '>' {
                next = scanner::next_token(r);
                match next {
                    scanner::Token::Name(to) => {
                        next = scanner::next_token(r);
                        Some(to)
                    }
                    _ => panic!("Name expected, {:?} found", next),
                }
            } else {
                None
            }
        }
        _ => {
            None
        }
    };

    (next, AST::Op(Atom {pars: v, src: src, dest: dest} ))
}

fn match_pipe(r : &mut reader::Reader) -> (scanner::Token, AST) {
    let expr1 = match_expr(r);
    if match expr1.0 {
        scanner::Token::Opr(c) => {
            if c == '|' {
                true
            } else {
                false
            }
        },
        _ => {
            false
        }
    } {
        let expr2 = match_pipe(r);
        let e1 = match expr1.1 {
            AST::Op(v) => v,
            _ => panic!("unexpected token {:?}", expr1.1)
        };
        (expr2.0, AST::Pipe(e1, Box::new(expr2.1)))
    } else {
        expr1
    }
}

fn match_and(r : &mut reader::Reader) -> AST {
    let pipe1 = match_pipe(r);
    let next_token : scanner::Token = 
        match pipe1.0 {
            scanner::Token::Empty => {
                scanner::next_token(r)
            }
            _ => pipe1.0
        };
    if match next_token {
        scanner::Token::Opr(c) => {
            if c == '&' {
                true
            } else {
                panic!("& required, {:} found", c);
            }
        },
        scanner::Token::EOF => {
            false
        }
        _ => {
            panic!("Opr(&) required, {:?} found", next_token);
        }
    } {
        AST::And(Box::new(pipe1.1), Box::new(match_and(r)))
    } else {
        pipe1.1
    }
}

pub fn ast_gen(r : &mut reader::Reader) -> AST {
    match_and(r)
}

/*
pub mod checker {
    use std::collections::HashMap;
    use super::parser;
    
    pub fn type_check(ast : parser::AST, vf : & HashMap<String, parser::Type>) -> parser::Type {
        match ast {
            parser::AST::Num(_) => return parser::Type::NumT,
            parser::AST::Bool(_) => return parser::Type::BoolT,
            parser::AST::Plus(l, r) => {
                let left = type_check(*l, vf);
                let right = type_check(*r, vf);
                if left != parser::Type::NumT || right != parser::Type::NumT {
                    panic!("Type is not NumT, {:?} != {:?}", left, right);
                }
                return left;
            },
            parser::AST::Mult(l, r) => {
                let left = type_check(*l, vf);
                let right = type_check(*r, vf);
                if left != parser::Type::NumT || right != parser::Type::NumT {
                    panic!("Type is not NumT, {:?} != {:?}", left, right);
                }
                return left;
            },
            parser::AST::Equ(l, r) => {
                let left = type_check(*l, vf);
                let right = type_check(*r, vf);
                if left != right {
                    panic!("Type not equal, {:?} != {:?}", left, right);
                }
                return parser::Type::BoolT;
            },
            parser::AST::If(cond, if_true, if_false) => {
                let c = type_check(*cond, vf);
                if c != parser::Type::BoolT {
                    panic!("If condition should be boolean, found type {:?}", c);
                }
                let i_t = type_check(*if_true, vf);
                let i_f = type_check(*if_false, vf);
                if i_t != i_f {
                    panic!("Type not equal, {:?} != {:?}", i_t, i_f);
                }
                return i_t;
            },
            parser::AST::Id(s) => {
                match vf.get(&s) {
                    Some(t) => return t.clone(),
                    None => panic!("Variable not found, {:?}", s),
                }
            },
            parser::AST::App(f, n) => {
                let fun_type = type_check(*f, vf);
                let in_type = type_check(*n, vf);
                match fun_type {
                    parser::Type::FunT(a, r) => {
                        if *a != in_type {
                            panic!("Function argument is invalid type {:?} != {:?}", a, in_type);
                        } else {
                            return *r;
                        }
                    },
                    _ => panic!("Needs to have a function type: {:?}", fun_type),
                }
            },
            parser::AST::Fd(name, i_t, r_t, stmt) => {
                let vf_ = &mut vf.clone();
                vf_.insert(name, i_t.clone());
                let stmt_type = type_check(*stmt, vf_);
                if stmt_type != r_t {
                    panic!("Statement type does not match return type {:?} != {:?}", r_t, stmt_type);
                }
                return parser::Type::FunT(Box::new(i_t), Box::new(r_t));
            }
            parser::AST::Error => {
                panic!("Error Type found");
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_email_examples() {
        let mut testings = reader::Reader::from_file(String::from("example.ty"));
        assert_eq!(checker::type_check(parser::ast_gen(&mut testings), & HashMap::new()), parser::Type::BoolT);
        assert_eq!(checker::type_check(parser::ast_gen(&mut testings), & HashMap::new()), parser::Type::FunT(Box::new(parser::Type::NumT), Box::new(parser::Type::NumT)));
        assert_eq!(checker::type_check(parser::ast_gen(&mut testings), & HashMap::new()), parser::Type::NumT);
        assert_eq!(checker::type_check(parser::ast_gen(&mut testings), & HashMap::new()), parser::Type::BoolT);
    }

    #[test]
    fn test_add() {
        let mut testing = reader::Reader::from_string(String::from("plusC(numC(3), numC(2))"));
        let ast = parser::ast_gen(&mut testing);
        assert_eq!(ast, parser::AST::Plus(Box::new(parser::AST::Num(3)), Box::new(parser::AST::Num(2))));
        assert_eq!(checker::type_check(ast, & HashMap::new()), parser::Type::NumT);
    }

    #[test]
    fn test_mult() {
        let mut testing = reader::Reader::from_string(String::from("multC(numC(3), numC(2))"));
        let ast = parser::ast_gen(&mut testing);
        assert_eq!(ast, parser::AST::Mult(Box::new(parser::AST::Num(3)), Box::new(parser::AST::Num(2))));
        assert_eq!(checker::type_check(ast, & HashMap::new()), parser::Type::NumT);
    }

    #[test]
    fn test_eq() {
        let mut testing = reader::Reader::from_string(String::from("eqC(falseC, trueC)"));
        let ast = parser::ast_gen(&mut testing);
        assert_eq!(ast, parser::AST::Equ(Box::new(parser::AST::Bool(false)), Box::new(parser::AST::Bool(true))));
        assert_eq!(checker::type_check(ast, & HashMap::new()), parser::Type::BoolT);
    }
}
*/