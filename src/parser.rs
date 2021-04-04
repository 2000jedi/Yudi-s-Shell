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
    None,
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
            = ws:escape()+ { 
                ws.join("")
             } / expected!("invalid input")

        rule escape() -> String
            = w:$(['a'..='z' | 'A'..='Z' | '0'..='9' | '.' | ',' | '_' | '-' | '/' | '=']+) {
                String::from(w)
            } / "\"" w:$((!['"'][_])*) "\"" {
                String::from(w)
            } / "'" w:$((!['\''][_])*) "'" {
               String::from(w)
            } / "`" w:$((!['`'][_])*) "`" {
                replace_exe(w.to_string())
            } / "$(" w:$((![')'][_])*) ")" {
                replace_exe(w.to_string())
            } / "\\" w:$([_]) {
                String::from(w)
            }

        rule ws() = quiet!{[' ' | '\t']+}
    }
}

pub fn ast_gen(s : String) -> AST {
    match shell_parser::outer(s.as_str()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e);
            AST::None
        }
    }
}

pub fn replace_exe(s : String) -> String {
    // Replace `s` with the stdout content from xxx.
    let args: Vec<&str> = s.split(" ").collect();
    let proc_name = args.get(0).unwrap();
    let proc_output = subprocess::Exec::cmd(proc_name).args(&args[1..])
        .stdout(subprocess::Redirection::Pipe).capture();
    let mut proc_output = match proc_output {
        Ok(val) => val.stdout_str(),
        Err(err) => {
            eprintln!("{}", err);
            return String::new();
        }
    };
    if proc_output.ends_with('\n') {
        proc_output.pop();
        if proc_output.ends_with('\r') {
            proc_output.pop();
        }
    }
    return proc_output;
}
