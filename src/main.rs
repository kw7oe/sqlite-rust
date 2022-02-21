use serde::{Deserialize, Serialize};
use std::{io::Write, process::exit};

#[derive(Debug)]
enum MetaCommand {
    // Success,
    Unrecognized,
}

#[derive(Debug)]
enum StatementType {
    Select,
    Insert,
}

#[derive(Debug)]
struct Statement {
    statement_type: StatementType,
}

#[macro_use]
extern crate serde_big_array;
big_array! {
    BigArray;
    32, 255
}

const USERNAME_SIZE: usize = 32;
const EMAIL_SIZE: usize = 255;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Row {
    id: u32,
    #[serde(with = "BigArray")]
    username: [u8; USERNAME_SIZE],
    #[serde(with = "BigArray")]
    email: [u8; EMAIL_SIZE],
}

fn main() -> std::io::Result<()> {
    let mut buffer = String::new();

    loop {
        print_prompt();
        std::io::stdin().read_line(&mut buffer)?;

        let input = buffer.trim();

        if input.starts_with(".") {
            match handle_meta_command(&input) {
                MetaCommand::Unrecognized => println!("Unrecognized command '{input}'."),
            }
        }

        match prepare_statement(&input) {
            Ok(statement) => execute_statement(&statement),
            Err(_reason) => println!("Unrecognized keyword at start of '{input}'."),
        }

        println!("Executed.");
        buffer.clear();
    }
}

fn print_prompt() {
    print!("db > ");
    let _ = std::io::stdout().flush();
}

fn handle_meta_command(command: &str) -> MetaCommand {
    if command.eq(".exit") {
        exit(0)
    } else {
        return MetaCommand::Unrecognized;
    }
}

fn prepare_statement(input: &str) -> Result<Statement, &str> {
    if input.starts_with("select") {
        return Ok(Statement {
            statement_type: StatementType::Select,
        });
    }

    if input.starts_with("insert") {
        return Ok(Statement {
            statement_type: StatementType::Insert,
        });
    }

    return Err("unrecognized statement");
}

fn execute_statement(statement: &Statement) {
    match statement.statement_type {
        StatementType::Select => {
            println!("do select")
        }
        StatementType::Insert => {
            let u = "apple";
            let m = "joe@apple.com";
            let mut username: [u8; USERNAME_SIZE] = [0; USERNAME_SIZE];
            let mut email: [u8; EMAIL_SIZE] = [0; EMAIL_SIZE];

            let mut index = 0;
            for c in u.bytes() {
                username[index] = c;
                index += 1;
            }

            index = 0;
            for c in m.bytes() {
                email[index] = c;
                index += 1;
            }

            let row = Row {
                id: 32,
                username,
                email,
            };

            let bytes = bincode::serialize(&row).unwrap();

            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("data.db")
                .unwrap();

            let _ = file.write(&bytes);
            println!("do insert")
        }
    }
}
