extern crate subprocess;

use super::parser;
use super::job_manager;
use std::env;
use std::path::Path;
use std::fs::File;
use std::sync::Mutex;
use std::collections::HashMap;

use subprocess::{Pipeline, Exec, Redirection, Popen, ExitStatus};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref FG_JOBS : Mutex<Vec<u32>> = Mutex::new(vec![]);
    pub static ref JOBS : Mutex<job_manager::Jobs> = Mutex::new(job_manager::Jobs::new());
    pub static ref ALIAS : Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

pub enum Inst {
    E(Exec),
    P(Pipeline),
    N,            // null
}

fn wait(jobs : Vec<Inst>) -> ExitStatus {
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
            return ExitStatus::Exited(255);
        }
    }

    let mut exit_status : ExitStatus = ExitStatus::Exited(0);
    for mut p in procs {
        exit_status = match p.wait() {
            Ok(val) => val,
            Err(err) => {
                println!("{}", err);
                continue;
            }
        };
    }

    return exit_status;
}

const BUILTIN : [&str; 6] = ["alias", "cd", "jobs", "fg", "bg", "exit"];

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
        "alias" => {
            match atom.pars.len() {
                1 => {
                    // show all aliases
                    match ALIAS.lock() {
                        Ok(a) => {
                            for (first, second) in &*a {
                                println!("{}={}", first, second);
                            }
                        }
                        Err(e) => {
                            eprintln!("{}", e);
                        }
                    }
                }
                2 => {
                    let arg : Vec<&str> = atom.pars.get(1).unwrap().split("=").collect();
                    match arg.len() {
                        1 => {
                            match ALIAS.lock() {
                                Ok(a) => {
                                    let first = &arg.get(0).unwrap().to_string();
                                    if a.contains_key(first) {
                                        println!("{}={}", first, a[first]);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("{}", e);
                                }
                            }
                        }
                        2 => {
                            match ALIAS.lock() {
                                Ok(mut a) => {
                                    let first = arg.get(0).unwrap().to_string();
                                    let second = arg.get(1).unwrap().to_string();
                                    if a.contains_key(&first) {
                                        *a.get_mut(&first).unwrap() = second;
                                    } else {
                                        a.insert(first, second);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("{}", e);
                                }
                            }
                        }
                        _ => {
                            eprintln!("alias: more than one equal operator is detected");
                        }
                    }
                }
                _ => {
                    eprintln!("alias: 0 or 1 argument required, {} received", atom.pars.len());
                }
            }
        }
        _ => {
            unreachable!();
        }
    }
}

fn create(v : parser::Atom, bg : bool) -> Option<Exec> {
    let mut proc_name = v.pars.get(0).unwrap().to_string();
    match ALIAS.lock() {
        Ok(a) => {
            if a.contains_key(&proc_name) {
                proc_name = a[&proc_name].clone();
                // TODO: this part should be modified
                proc_name = parser::replace_exe(proc_name);
            }
        }
        Err(_) => {}
    }

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

    if bg {
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

fn walk_pipe(procs : Vec<parser::Atom>, bg : bool) -> Vec<Inst> {
    let mut inst : Inst = Inst::N;
    for p in procs {
        let proc = match create(p, bg) {
            Some(v) => v,
            None    => { return vec![]; }
        };

        inst = match inst {
            Inst::E(proc2) => Inst::P(proc2 | proc),
            Inst::P(pipe)  => Inst::P(pipe  | proc),
            Inst::N        => Inst::E(proc),
        };
    }
    
    return vec![inst];
}

fn walk(ast : parser::AST) -> ExitStatus {
    match ast {
        parser::AST::Fg(v) => {
            wait(walk_pipe(v, false))
        }
        parser::AST::Bg(v) => {
            wait(walk_pipe(v, true))
        }
        parser::AST::BinOp(first, second, opr) => {
            match opr {
                parser::Op::AND => {
                    let proc1 = walk(*first);
                    if proc1.success() {
                        let proc2 = walk(*second);
                        return proc2;
                    }
                    return proc1;
                }
                parser::Op::OR => {
                    let proc1 = walk(*first);
                    if ! proc1.success() {
                        let proc2 = walk(*second);
                        return proc2;
                    }
                    return proc1;
                }
                parser::Op::SEQ => {
                    walk(*first);
                    walk(*second)
                }
            }
        }
        parser::AST::None => ExitStatus::Exited(0),
    }
}

pub fn run(ast : parser::AST) {
    let procs = walk(ast);
    if ! procs.success() {
        println!("[Process exited with {:?}]", procs);
    }
}
