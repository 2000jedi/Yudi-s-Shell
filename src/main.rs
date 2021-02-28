extern crate signal_hook;

use std::env;
use std::io::{self, Write};
use signal_hook::consts::signal;
use signal_hook::iterator::Signals;
use signal_hook::low_level;

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
            std::process::exit(0);
        }
    };
    let mut read = reader::Reader::from_file(file_path);
    let ast = parser::ast_gen(&mut read);
    shell::run(ast);

    while read.has_next() {
        println!("error: unused token \"{:?}\"", scanner::next_token(&mut read));
    }

    println!("completed");
    std::process::exit(0);
}

fn main() {
    /*
    Signal Handling
    */

    let mut signals = Signals::new(vec![signal::SIGTSTP, signal::SIGCHLD, signal::SIGINT]).unwrap();
    let signal_handler = signals.handle();
    let main_proc = std::thread::spawn(run);

    for signal in signals.forever() {
        match signal {
            signal::SIGCHLD => {
                match low_level::emulate_default_handler(signal) {
                    // FIXME: this works even without default handler
                    Ok(_) => {},
                    Err(e) => eprintln!("{}", e)
                };
                let mut jobs_lock = shell::JOBS.lock();
                loop {
                    match jobs_lock {
                        Ok(mut jobs) => {
                            jobs.refresh();
                            break;
                        }
                        Err(_) => jobs_lock = shell::JOBS.lock(),
                    }
                }
            }
            signal::SIGINT => {
                // FIXME: bg children still receive SIGINT
                println!("SIGINT received");
            }
            signal::SIGTSTP => {

            }
            _ => unreachable!(),
        }
        
    }

    signal_handler.close();
}
