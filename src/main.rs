use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;

#[derive(Debug)]
struct Token {
}

fn tokenize(source: &String) -> Result<Vec<Token>, String> {
    unimplemented!();
}

fn run(source: &String) -> Result<(), String> {
    let tokens = try!(tokenize(source));
    for token in tokens {
        println!("{:?}", token);
    }
    Ok(())
}

fn run_file(file_name: &String) -> Result<(), String> {
    match File::open(file_name) {
        Err(_) => {
            Err("Error opening file".into()) // TODO: add context
        }
        Ok(mut file) => {
            let mut source = String::new();
            match file.read_to_string(&mut source) {
                Err(_) => {
                    Err("Error reading file".into()) // TODO: add context
                }
                Ok(_) => run(&source),
            }
        }
    }
}

fn run_prompt() -> Result<(), String> {
    loop {
        print!("> ");
        io::stdout().flush();
        let mut source = String::new();
        io::stdin().read_line(&mut source);
        try!(run(&source));
        ()
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let result = match args.len() {
        // The first argument is the program name
        1 => run_prompt(),
        2 => run_file(&args[1]),
        _ => {
            println!("Usage: rulox [script]");
            Ok(())
        }
    };
    let exit_code = match result {
        Ok(_) => 0,
        Err(e) => {
            println!("{}", e);
            1
        }
    };
    std::process::exit(exit_code)
}
