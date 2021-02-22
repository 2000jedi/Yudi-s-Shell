// use std::collections::HashMap;
use std::env;

pub mod reader;
pub mod scanner;
pub mod parser;

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = match args.get(1) {
        Some(t) => t.to_string(),
        None => panic!("use file path as program parameter"),
    };
    let mut read = reader::Reader::from_file(file_path);
    let ast = parser::ast_gen(&mut read);
    println!("{:?}", ast);
    // let tc = checker::type_check(ast, & HashMap::new());
    // println!("{:?}", tc);

    println!("Showing unused tokens:");
    while read.has_next() {
        println!("{:?}", scanner::next_token(&mut read));
    }
    //println!("{:?}", parser::ast_gen(&mut read));
}
