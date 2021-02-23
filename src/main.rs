// use std::collections::HashMap;
use std::env;

pub mod reader;
pub mod scanner;
pub mod parser;
pub mod shell;

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = match args.get(1) {
        Some(t) => t.to_string(),
        None => panic!("use file path as program parameter"),
    };
    let mut read = reader::Reader::from_file(file_path);
    let ast = parser::ast_gen(&mut read);
    println!("{:?}", ast);

    shell::run(ast);

    while read.has_next() {
        println!("error: unused token \"{:?}\"", scanner::next_token(&mut read));
    }
}
