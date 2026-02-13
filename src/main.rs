// use std::io::{self, Write};

// fn main() {
//     // TODO: Uncomment the code below to pass the first stage
//     let mut command : String = String::new();
//     // let mut followup : String = String::new();
//     loop {
//         command.clear();
//         print!("$ ");
//         io::stdout().flush().unwrap();
//         io::stdin().read_line(&mut command).unwrap();
//         if command.trim().len() == 0 {continue;}
//         else if command.trim() == "exit" {break;}
//         else if command[0..4].trim() == "echo" {
//             print!("{}\n", command[4..].trim());
//         } 
//         else {
//             print!("{}: command not found\n", command.trim());
//         }
//     }
// }


#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();

        let parts = command.split_whitespace().collect::<Vec<_>>();
        if parts.is_empty() {
            println!("{}: command not found", command.trim());
            continue;
        }

        match parts[0] {
            "echo" => println!("{}", &command[4..].trim()),
            "exit" => break,
            _ => println!("{}: command not found", command.trim()),
        }
    }
}