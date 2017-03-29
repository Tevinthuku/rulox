mod scanner;
mod ast;
mod pretty_printer;
mod parser;

extern crate itertools;

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use pretty_printer::PrettyPrint;

#[derive(Debug)]
enum Error{
    IoError(String),
    ScannerError(scanner::ScannerError),
    ParserError(parser::ParseError)
}

fn run(source: &String) -> Result<(), Error> {
    match scanner::scan(source){
        Ok(tokens) =>{
            match parser::parse(tokens) {
                Ok(expr) => {
                    println!("{:?}", expr.pretty_print());
                    Ok(())
                },
                Err(err) => Err(Error::ParserError(err))
            }
        }
        Err(err) => Err(Error::ScannerError(err))
    }
}

fn run_file(file_name: &String) -> Result<(), Error> {
    match File::open(file_name) {
        Err(_) => {
            Err(Error::IoError("Error opening file".into())) // TODO: add context
        }
        Ok(mut file) => {
            let mut source = String::new();
            match file.read_to_string(&mut source) {
                Err(_) => {
                    Err(Error::IoError("Error reading file".into())) // TODO: add context
                }
                Ok(_) => run(&source),
            }
        }
    }
}

fn run_prompt() -> Result<(), Error> {
    println!("Rulox - A lox interpreter written in Rust");
    let _ = io::stdout().flush(); //TODO: is this okay?
    loop {
        print!("> ");
        let _ = io::stdout().flush();
        let mut source = String::new();
        let _ = io::stdin().read_line(&mut source);
        // TODO: add a way to exit
        try!(run(&source))
        // TODO: report syntax errors to the user
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
            println!("{:?}", e);
            1
        }
    };
    std::process::exit(exit_code)
}
