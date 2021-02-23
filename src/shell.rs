extern crate subprocess;

use super::parser;
use std::fs::File;
use subprocess::{Pipeline, Exec, Redirection};

enum Inst {
    E(Exec),
    P(Pipeline),
    N,            // null
}

fn wait(procs : Vec<Inst>) {
    for p in procs {
        let exit_status = match p {
            Inst::E(e) => e.join().unwrap(),
            Inst::P(p) => p.join().unwrap(),
            Inst::N    => panic!("empty process"),
        };
        if ! exit_status.success() {
            println!("{:?}", exit_status);
        }
    }
}

fn create(v : parser::Atom) -> Exec {
    let mut proc = Exec::cmd(v.pars.get(0).unwrap()).args(&v.pars[1..]);

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

    return proc;
}

fn walk(ast : parser::AST, stdin : Inst) -> Vec<Inst> {
    match ast {
        parser::AST::Op(v) => {
            let proc = create(v);

            match stdin {
                Inst::E(proc2) => vec![(Inst::P(proc2 | proc))],
                Inst::P(pipe)  => vec![(Inst::P(pipe  | proc))],
                Inst::N        => vec![ Inst::E(proc)],
            }
        }
        parser::AST::Pipe(first, second) => {
            let proc1 = create(first);

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
        parser::AST::Error => {
            panic!("an error occured in command-line");
        }
    }
}

pub fn run(ast : parser::AST) {
    let procs = walk(ast, Inst::N);
    wait(procs);
}