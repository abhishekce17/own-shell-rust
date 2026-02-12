use std::io::{self, Write};

fn main() {
    // TODO: Uncomment the code below to pass the first stage
    let mut command : String = String::new();
    loop {
        command.clear();
        print!("$ ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut command).unwrap();
        if command.trim() == "exit" {break;};
        println!("{}: command not found", command.trim());
    }
}
