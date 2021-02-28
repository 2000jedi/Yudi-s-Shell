extern crate libc;

use std::env;
use std::io::{self, Write};

pub mod reader;
pub mod scanner;
pub mod parser;
pub mod shell;
pub mod job_manager;

fn repl() {
    'main: loop {
        let path = String::from(
            env::current_dir().unwrap().as_path().file_stem().unwrap().to_str().unwrap());
        print!("{} ➜ ", path);
        io::stdout().flush().unwrap();
        let mut line = String::new();
        let mut buf = String::new();
        loop {
            match io::stdin().read_line(&mut buf) {
                Ok(n) => {
                    if n == 0 {
                        break 'main;
                    }
                    if n == 1 {
                        if line.len() == 0 {
                            print!("{} ➜ ", path);
                            io::stdout().flush().unwrap();
                        } else {
                            print!("> ");
                            io::stdout().flush().unwrap();
                        }
                        continue;
                    }
                    if buf.as_bytes()[n - 2] != '\\' as u8 {
                        line = line + &buf;
                        break;
                    } else {
                        line = line + &buf[..n-2];
                    }
                    buf = String::new();
                    print!("> ");
                    io::stdout().flush().unwrap();
                }
                Err(e) => {
                    panic!("unexpected end-of-input: {}", e);
                }
            }
        }

        if line.ends_with('\n') {
            /* truncate terminating newline symbols */
            line.pop();
            if line.ends_with('\r') {
                line.pop();
            }
        }

        let mut read = reader::Reader::from_string(line);
        let ast = parser::ast_gen(&mut read);
        println!("{:?}", ast);
        shell::run(ast);
    }
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
    let mut read = reader::Reader::from_file(file_path);
    let ast = parser::ast_gen(&mut read);
    shell::run(ast);

    while read.has_next() {
        println!("error: unused token \"{:?}\"", scanner::next_token(&mut read));
    }

    println!("completed");
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
