use std::env;
use std::io::{self, Write};

pub mod reader;
pub mod scanner;
pub mod parser;
pub mod shell;
pub mod job_manager;

fn repl() {
    loop {
        let path = String::from(
            env::current_dir().unwrap().as_path().file_stem().unwrap().to_str().unwrap());
        print!("{} ➜ ", path);
        io::stdout().flush().unwrap();
        let mut line = String::new();
        let mut buf = String::new();
        loop {
            match io::stdin().read_line(&mut buf) {
                Ok(n) => {
                    if n <= 1 {
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

        let mut read = reader::Reader::from_string(line);
        let ast = parser::ast_gen(&mut read);
        println!("{:?}", ast);
        shell::run(ast);
    }
    
    // env::set_current_dir()
}

fn main() {
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
}
