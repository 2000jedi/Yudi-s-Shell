use super::scanner;
use super::reader;

#[derive(Debug, PartialEq)]
pub struct Atom {
    pub pars : Vec<String>,     // parameters of the process
    pub src  : Option<String>,  // direct pipe from a file
    pub dest : Option<String>,  // direct pipe to a file
    pub isbg : bool,            // run the process in background
}

#[derive(Debug, PartialEq)]
pub enum AST {
    Op(Atom),                 // exec < src > dest
    And(Box<AST>, Box<AST>),  // a & b
    Pipe(Atom, Box<AST>),     // a | b
    Empty,
    Error,
}

fn match_expr(r : &mut reader::Reader) -> (scanner::Token, AST) {
    let mut next = scanner::next_token(r);
    let mut v = Vec::new();
    match next {
        scanner::Token::Name(name) => {
            v.push(name);
        }
        scanner::Token::EOF => {
            return (scanner::Token::Empty, AST::Empty);
        }
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

    let bg = match next {
        scanner::Token::Opr(c) => {
            if (c == '&') && (! r.has_next()) {
                next = scanner::Token::EOF;
                true
            } else {
                false
            }
        }
        _ => false,
    };

    (next, AST::Op(Atom {pars: v, src: src, dest: dest, isbg: bg} ))
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
