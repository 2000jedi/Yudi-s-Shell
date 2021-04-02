extern crate libc;
extern crate peg;

use std::env;
use rustyline::error::ReadlineError;
use rustyline::Editor;

pub mod parser;
pub mod shell;
pub mod job_manager;
pub mod utils;

fn repl() {
    let mut reader = Editor::<()>::new();
    match reader.load_history(&(utils::home_dir() + "/.rsh_history")) {
        Ok(_) => {}
        Err(e) => {
            println!("load_history() error: {}", e);
            reader.save_history(&(utils::home_dir() + "/.rsh_history")).unwrap();
        }
    };

    'main: loop {
        let path = String::from(env::current_dir().unwrap().as_path().file_stem().unwrap().to_str().unwrap());
        let mut readline = reader.readline(&(path + " ➜ "));

        let mut input = String::new();
        loop {
            
            match readline {
                Ok(mut line) => {
                    while line.ends_with(' ') {
                        line.pop();
                    }
                    if line.len() == 0 {
                        continue 'main;
                    }
                    input += &line;
                    if line.as_bytes()[line.len() - 1] == '\\' as u8 {
                        readline = reader.readline(" > ");
                    } else {
                        break;
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("");
                    continue 'main;
                }
                Err(ReadlineError::Eof) => {
                    break 'main;
                }
                Err(err) => {
                    eprintln!("{}", err);
                    continue 'main;
                }
            }
        }

        reader.add_history_entry(input.as_str());
        let ast = parser::ast_gen(input);
        println!("{:?}", ast);
        shell::run(ast);
    }
    reader.append_history(&(utils::home_dir() + "/.rsh_history")).unwrap();
}

fn run() {
    /*
    Command Interpreting
    */
    let args: Vec<String> = env::args().collect();
    let file_path = match args.get(1) {
        Some(t) => t.to_string(),
        None => {
            repl();
            return;
        }
    };
    let input = match std::fs::read_to_string(file_path) {
        Ok(t) => t,
        Err(_e) => panic!("File Not Found"),
        };
    let ast = parser::ast_gen(input);
    shell::run(ast);
}

fn main() {
    /*
    Signal Handling
    */
    unsafe {
        libc::signal(libc::SIGINT, get_int_handler());
        libc::signal(libc::SIGTSTP, get_stop_handler());
        libc::signal(libc::SIGCHLD, get_chld_handler());
    }

    run();
}

extern fn int_handler(_: libc::c_int) {
    match shell::FG_JOBS.lock() {
        Ok(fg) => {
            for job in &*fg {
                unsafe {
                    libc::kill(-(*job as i32), libc::SIGINT);
                }
            }
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}

extern fn stop_handler(_: libc::c_int) {
    match shell::FG_JOBS.lock() {
        Ok(fg) => {
            for job in &*fg {
                unsafe {
                    libc::kill(-(*job as i32), libc::SIGTSTP);
                }
            }
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}

extern fn chld_handler(_: libc::c_int) {
    match shell::JOBS.lock() {
        Ok(mut jobs) => {
            jobs.refresh();
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}

fn get_int_handler() -> libc::sighandler_t {
    int_handler as extern fn(libc::c_int) as *mut libc::c_void as libc::sighandler_t
}

fn get_stop_handler() -> libc::sighandler_t {
    stop_handler as extern fn(libc::c_int) as *mut libc::c_void as libc::sighandler_t
}

fn get_chld_handler() -> libc::sighandler_t {
    chld_handler as extern fn(libc::c_int) as *mut libc::c_void as libc::sighandler_t
}
