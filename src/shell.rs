extern crate subprocess;

use super::parser;
use subprocess::{Pipeline, Exec};

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

fn walk(ast : parser::AST, stdin : Inst) -> Vec<Inst> {
    match ast {
        parser::AST::Op(v) => {
            let proc = Exec::cmd(v.pars.get(0).unwrap()).args(&v.pars[1..]);

            match stdin {
                Inst::E(proc2) => vec![(Inst::P(proc2 | proc))],
                Inst::P(pipe)  => vec![(Inst::P(pipe  | proc))],
                Inst::N        => vec![ Inst::E(proc)],
            }
        }
        parser::AST::Pipe(first, second) => {
            let proc1 = Exec::cmd(first.pars.get(0).unwrap()).args(&first.pars[1..]);

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