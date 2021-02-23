extern crate subprocess;

use super::parser;
use std::env;
use std::path::Path;
use std::fs::File;
use subprocess::{Pipeline, Exec, Redirection};

enum Inst {
    E(Exec),
    P(Pipeline),
    N,            // null
}

fn wait(procs : Vec<Inst>) {
    for p in procs {
        match p {
            Inst::E(e) => {
                let exit_status = e.join().unwrap();
                if ! exit_status.success() {
                    println!("{:?}", exit_status);
                }
            }
            Inst::P(p) => {
                let exit_status = p.join().unwrap();
                if ! exit_status.success() {
                    println!("{:?}", exit_status);
                }
            }
            Inst::N    => {}
        }
    }
}

const BUILTIN : [&str; 1] = ["cd"];

fn builtin(proc_name: String, args: &[String]) {
    if proc_name == "cd" {
        if args.len() != 1 {
            panic!("cd: 1 parameter expected, {} given", args.len());
        }
        let base_path = args.get(0).unwrap();
        env::set_current_dir(Path::new(base_path)).unwrap();
    }
}

fn create(v : parser::Atom) -> Option<Exec> {
    let proc_name = v.pars.get(0).unwrap();
    if BUILTIN.contains(&proc_name.as_str()) {
        builtin(proc_name.to_string(), &v.pars[1..]);
        return None;
    }

    let mut proc = Exec::cmd(proc_name).args(&v.pars[1..]);

    match v.src {
        Some(stdin) => {
            proc = proc.stdin(Redirection::File(File::open(stdin).unwrap()));
        }
        None => {}
    }

    match v.dest {
        Some(stdout) => {
            proc = proc.stdout(Redirection::File(File::create(stdout).unwrap()));
        }
        None => {}
    }

    return Some(proc);
}

fn walk(ast : parser::AST, stdin : Inst) -> Vec<Inst> {
    match ast {
        parser::AST::Op(v) => {
            let proc = match create(v){
                Some(v) => v,
                None    => return vec![],
            };

            match stdin {
                Inst::E(proc2) => vec![(Inst::P(proc2 | proc))],
                Inst::P(pipe)  => vec![(Inst::P(pipe  | proc))],
                Inst::N        => vec![ Inst::E(proc)],
            }
        }
        parser::AST::Pipe(first, second) => {
            let proc1 = match create(first){
                Some(v) => v,
                None    => return vec![],
            };

            let pipe = match stdin {
                Inst::E(proc2) => Inst::P(proc2 | proc1),
                Inst::P(pipe)  => Inst::P(pipe  | proc1),
                Inst::N        => Inst::E(proc1),
            };

            walk(*second, pipe)
        }
        parser::AST::And(first, second) => {
            let mut proc1 = walk(*first, Inst::N);
            let mut proc2 = walk(*second, Inst::N);
            proc1.append(&mut proc2);

            proc1
        }
        parser::AST::Empty => vec![],
        parser::AST::Error => panic!("an error occured in command-line"),
    }
}

pub fn run(ast : parser::AST) {
    let procs = walk(ast, Inst::N);
    wait(procs);
}