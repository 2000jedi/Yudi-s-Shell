use peg;

#[derive(Debug, PartialEq)]
pub struct Atom {
    pub pars : Vec<String>,     // parameters of the process
    pub src  : Option<String>,  // direct pipe from a file
    pub dest : Option<String>,  // direct pipe to a file
}

#[derive(Debug, PartialEq)]
pub enum Op {
    AND,  // &&
    OR,   // ||
    SEQ   // ; or &
}

#[derive(Debug, PartialEq)]
pub enum AST {
    Fg(Vec<Atom>),                // foreground command: exec < src > dest
    Bg(Vec<Atom>),                // background command: exec < src > dest &
    BinOp(Box<AST>, Box<AST>, Op),// a <op> b
}

peg::parser!{
    grammar shell_parser() for str {
        pub rule outer() -> AST = v:binop() { v }

        rule binop() -> AST = precedence! {
            x:(@) "&&" y:@ { AST::BinOp(Box::new(x), Box::new(y), Op::AND) }
            x:(@) "||" y:@ { AST::BinOp(Box::new(x), Box::new(y), Op::OR) }
            x:(@) ";" y:@ { AST::BinOp(Box::new(x), Box::new(y), Op::SEQ) }
            x:(@) "&" y:@ { AST::BinOp(Box::new(x), Box::new(y), Op::SEQ) }
            --
            ws()* p:pipes() "&" ![_] { AST::Bg(p) }
            ws()* p:pipes() &"&" !"&&" { AST::Bg(p) }
            ws()* p:pipes() { AST::Fg(p) }
        }

        rule pipes() -> Vec<Atom> = ps:process_redirect() ** (ws()* "|" ws()*) { ps }

        rule process_redirect() -> Atom
            = pars:process() src:pipe_in()? dest:pipe_out()? ws()* {
                Atom{pars, src, dest}
            } / pars:process() src:pipe_out()? dest:pipe_in()? ws()* {
                Atom{pars, src, dest}
            }

        rule pipe_in() -> String = ws()* "<" ws()* w:word() { w }
        rule pipe_out() -> String = ws()* ">" ws()* w:word() { w }
        rule process() -> Vec<String> = par:word() ws()* pars:word() ** ws() {
            let mut ps : Vec<String> = vec![par];
            for p in pars { ps.push(p); }
            ps
        }

        rule word() -> String
            = w:$(['a'..='z' | 'A'..='Z' | '0'..='9' | '.' | ',' | '_' | '`' | '-' | '\'' | '"' | '/' | '\\']+) { 
                // TODO: string manupulation
                String::from(w)
             } / expected!("invalid input")

        rule ws() = quiet!{[' ' | '\t']+}
    }
}

pub fn ast_gen(s : String) -> AST {
    shell_parser::outer(s.as_str()).unwrap()
}

pub fn replace_exe(s : String) -> String {
    // TODO: replace `xxx` with the stdout content from xxx.
    return s;
}
