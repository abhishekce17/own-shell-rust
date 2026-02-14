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

enum ShellBuiltins {
    ECHO,
    EXIT,
    TYPE
}

fn get_command(command: &str) -> Option<ShellBuiltins> {
    match command {
        "echo" => Some(ShellBuiltins::ECHO),
        "exit" => Some(ShellBuiltins::EXIT),
        "type" => Some(ShellBuiltins::TYPE),
        _ => None,
    }
}


fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();

        let parts: Vec<&str> = command.split_whitespace().collect::<Vec<_>>();
        if parts.is_empty() {
            println!("{}: command not found", command.trim());
            continue;
        }
        

        match get_command(&parts[0]) {
            Some(ShellBuiltins::ECHO) => println!("{}", &command[4..].trim()),
            Some(ShellBuiltins::EXIT) => break,
            Some(ShellBuiltins::TYPE) => match get_command(&parts[1]) {
                Some(_) => println!("{} is a shell builtin", parts[1]),
                _ => println!("{}: not found", parts[1]),
                
            },
            _ => println!("{}: command not found", parts[0]),
        }
    }
}