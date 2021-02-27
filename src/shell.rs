extern crate subprocess;

use super::parser;
use super::job_manager;
use std::env;
use std::path::Path;
use std::fs::File;
use std::sync::Mutex;

use subprocess::{Pipeline, Exec, Redirection};
use lazy_static::lazy_static;

enum Inst {
    E(Exec),
    P(Pipeline),
    N,            // null
}

fn wait(procs : Vec<Inst>) {
    for p in procs {
        match p {
            Inst::E(e) => {
                let exit_status = match e.join() {
                    Ok(val) => val,
                    Err(err) => {
                        println!("{}", err);
                        continue;
                    }
                };

                if ! exit_status.success() {
                    println!("[Process exited with {:?}]", exit_status);
                }
            }
            Inst::P(p) => {
                let exit_status = match p.join() {
                    Ok(val) => val,
                    Err(err) => {
                        println!("{}", err);
                        continue;
                    }
                };
                if ! exit_status.success() {
                    println!("[Process exited with {:?}]", exit_status);
                }
            }
            Inst::N    => {}
        }
    }
}

const BUILTIN : [&str; 4] = ["cd", "jobs", "fg", "bg"];

lazy_static! {
    static ref JOBS : Mutex<job_manager::Jobs> = Mutex::new(job_manager::Jobs::new());
}

fn builtin(proc_name: String, atom: parser::Atom) {
    if proc_name == "cd" {
        let args = &atom.pars[1..];
        if args.len() != 1 {
            panic!("cd: 1 parameter expected, {} given", args.len());
        }
        let base_path = args.get(0).unwrap();

        match env::set_current_dir(Path::new(base_path)) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", e);
            }
        }

        return;
    }
    if proc_name == "bg" {
        /*
        let proc_match_create = create(parser::Atom {
            pars: (&atom.pars[1..]).to_vec(),
            src:  atom.src,
            dest: atom.dest,
            isbg: atom.isbg,
        });

        let proc_match = match proc_match_create {
            Some(v) => v.detached().popen(),
            None => {
                eprintln!("{} is a builtin command", atom.pars[1]);
                return;
            }
        };

        let proc = match proc_match {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        };

        let mut jobs_lock = JOBS.lock();
        loop {
            match jobs_lock {
                Ok(mut j) => {
                    j.push(proc, atom.pars.join(" "));
                    // TODO: recycle zombie processes
                    break;
                }
                Err(_) => jobs_lock = JOBS.lock(),
            }
        }*/
        
        return;
    }

    if proc_name == "fg" {
        let mut jobs_lock = JOBS.lock();
        loop {
            match jobs_lock {
                Ok(_) => {
                    // TODO: move job to foreground by spinn-waiting it
                    break;
                }
                Err(_) => jobs_lock = JOBS.lock(),
            }
        }
    }

    if proc_name == "jobs" {
        let mut jobs_lock = JOBS.lock();
        loop {
            match jobs_lock {
                Ok(j) => {
                    j.print();
                    break;
                }
                Err(_) => jobs_lock = JOBS.lock(),
            }
        }

        return;
    }
}

fn create(v : parser::Atom) -> Option<Exec> {
    let proc_name = v.pars.get(0).unwrap();
    if BUILTIN.contains(&proc_name.as_str()) {
        builtin(proc_name.to_string(), v);
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

    if v.isbg {
        let bg_proc = match proc.detached().popen() {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{}", e);
                return None;
            }
        };

        let mut jobs_lock = JOBS.lock();
        loop {
            match jobs_lock {
                Ok(mut j) => {
                    j.push(bg_proc, v.pars.join(" "));
                    // TODO: recycle zombie processes
                    break;
                }
                Err(_) => jobs_lock = JOBS.lock(),
            }
        }
        return None;
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