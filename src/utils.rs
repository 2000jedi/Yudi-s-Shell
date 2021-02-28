use std::env;

pub fn home_dir() -> String {
    match env::var("HOME") {
        Ok(val) => val,
        Err(_) => {
            eprintln!("Error: environment variable `HOME` not found, defaulting to /home/");
            String::from("/home/")
        }
    }
}
