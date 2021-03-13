extern crate subprocess;

use super::parser;
use super::job_manager;
use std::env;
use std::path::Path;
use std::fs::File;
use std::sync::Mutex;

use subprocess::{Pipeline, Exec, Redirection, Popen};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref FG_JOBS : Mutex<Vec<u32>> = Mutex::new(vec![]);
    pub static ref JOBS : Mutex<job_manager::Jobs> = Mutex::new(job_manager::Jobs::new());
}

pub enum Inst {
    E(Exec),
    P(Pipeline),
    N,            // null
}

fn wait(jobs : Vec<Inst>) {
    let mut procs : Vec<Popen> = vec![];
    let mut pids : Vec<u32> = vec![];
    for j in jobs {
        match j {
            Inst::E(e) => {
                match e.detached().popen() {
                    Ok(proc) => {
                        pids.push(proc.pid().unwrap());
                        procs.push(proc);
                    }
                    Err(err) => eprintln!("{}", err)
                }
            }
            Inst::P(p) => {
                match p.popen() {
                    Ok(proc) => {
                        for p in proc {
                            pids.push(p.pid().unwrap());
                            procs.push(p);
                        }
                    }
                    Err(err) => eprintln!("{}", err)
                }
            }
            Inst::N    => {}
        }
    }

    match FG_JOBS.lock() {
        Ok(mut fg) => {
            *fg = pids;
        }
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    }

    for mut p in procs {
        let exit_status = match p.wait() {
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
}

const BUILTIN : [&str; 5] = ["cd", "jobs", "fg", "bg", "exit"];

fn builtin(proc_name: String, atom: parser::Atom) {
    match proc_name.as_str() {
        "cd" => {
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
        }
        "bg" => {

        }
        "fg" => {
            let job : Option<job_manager::Job> = match JOBS.lock() {
                Ok(mut jobs) => {
                    let job_id = match atom.pars.get(1) {
                        Some(v) => match v.parse::<u32>() {
                            Ok(v) => v,
                            Err(_) => {
                                println!("fg: job not found: {}", v);
                                0
                            }
                        }
                        None => {
                            eprintln!("fg: no current job");
                            0
                        }
                    };
                    if job_id == 0 {
                        return;
                    }

                    let mut ind : usize = 0;
                    for job in &jobs.jobs {
                        if job.jid == job_id {
                            break;
                        }
                        ind += 1;
                    }

                    if ind >= jobs.jobs.len() {
                        println!("fg: job not found: {}", job_id);
                        return;
                    }

                    Some(jobs.jobs.remove(ind))
                }
                Err(e) => {
                    eprintln!("{}", e);
                    None
                }
            };

            match job {
                Some(mut job) => {
                    println!("[{}] {} running {}", job.jid, job.pid, job.cmd);
                    match FG_JOBS.lock() {
                        Ok(mut fg) => {
                            *fg = vec![job.pid];
                        }
                        Err(e) => {
                            eprintln!("{}", e);
                            return;
                        }
                    }
                    match job.proc.wait() {
                        Ok(val) => {
                            println!("[{}] {} {:?} {}",job.jid, job.pid, val, job.cmd);
                            return;
                        }
                        Err(e) => {
                            println!("error: {}", e);
                        }
                    }
                }
                None => {}
            }
        }
        "jobs" => {
            match JOBS.lock() {
                Ok(j) => {
                    j.print();
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
        }
        "exit" => {
            std::process::exit(0);
        }
        _ => {
            unreachable!();
        }
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

        match JOBS.lock() {
            Ok(mut j) => {
                j.push(bg_proc, v.pars.join(" "));
                let job = j.back();
                println!("[{}] {} {}", job.jid, job.pid, job.cmd);
            }
            Err(e) => {
                eprintln!("{}", e);
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
