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
use std::{env, path::PathBuf};
use std::process::Command;
use std::path::{Path};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;


enum ShellBuiltins {
    ECHO,
    EXIT,
    TYPE,
    PWD,
    CD
}

fn get_command(command: &str) -> Option<ShellBuiltins> {
    match command {
        "echo" => Some(ShellBuiltins::ECHO),
        "exit" => Some(ShellBuiltins::EXIT),
        "type" => Some(ShellBuiltins::TYPE),
        "pwd" => Some(ShellBuiltins::PWD),
        "cd" => Some(ShellBuiltins::CD),
        _ => None,
    }
}

fn is_executable_command(command: &str) -> Option<PathBuf> {
    if let Some(path_env) = env::var_os("PATH"){
        // println!("{}: searching in PATH", path_env.to_string_lossy());
        let exe_array: [&str; 4] = [ "", "exe" , "bat" , "cmd" ];
        for dir in env::split_paths(&path_env){
            let full_path = dir.join(command);
            if exe_array.iter().any(|&ext| {
                if ext.is_empty() {
                    if full_path.exists() && full_path.is_file() {
                        #[allow(unused_variables)]
                        if let Ok(metadata) = full_path.metadata() {
                            #[cfg(unix)]
                            {
                                return metadata.permissions().mode() & 0o111 != 0;
                            }
                            #[cfg(not(unix))]
                            {
                                return false;
                            }
                        }else {
                            return false;
                        }
                    }else {
                        return false;
                    }
                } else {
                    full_path.with_extension(ext).exists()
                }
            }) {
                // println!("{} is {}", command, full_path.display());
                return Some(full_path);
            }
        }
        return None;
    }
    return None;
}

fn parse_args(input: &str) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();
    let mut current_arg: String = String::new();
    let mut quote_char: Option<char> = None; // None means we are NOT in quotes

    for c in input.chars() {
        match (quote_char, c) {
            (Some(q), c) if q == c => {
                quote_char = None;
            }
            (None, '\'' | '"') => {
                quote_char = Some(c);
            }
            (None, ' ') => {
                if !current_arg.is_empty() {
                    args.push(current_arg.clone());
                    current_arg.clear();
                }
            }
            (_, c) => {
                current_arg.push(c);
            }
        }
    }
    
    // Push the final argument if it exists
    if !current_arg.is_empty() {
        args.push(current_arg);
    }
    
    args
}

fn set_current_dit(parent_path : &Path, path: &str){
    let new_path: PathBuf = parent_path.join(path);
    if new_path.exists(){
        env::set_current_dir(new_path).unwrap();
    }else {
        println!("cd: {}: No such file or directory", &path);
    }
}

fn cd_functionality(parts : &Vec<String>){
                if parts.len() < 2 {
                } else if parts.len() > 2 {
                    println!("too many arguments");
                } else {
                    if parts[1].starts_with("/") {
                        let new_path = Path::new(&parts[1]);
                        if new_path.is_absolute() && new_path.exists() {
                        env::set_current_dir(&parts[1]).unwrap();
                        return;
                    }else {
                        println!("cd: {}: No such file or directory", &parts[1]);
                        return;
                    }
                }else {
                        let new_dir: Vec<&str>= parts[1].split("/").collect();
                        match new_dir[0] {
                            "~" => env::set_current_dir(env::home_dir().unwrap()).unwrap(),
                            ".." => {
                                if let Some(parent_dir) = env::current_dir().unwrap().parent(){
                                    env::set_current_dir(parent_dir).unwrap();
                                    if new_dir[1..].len() > 0{ 
                                        // env::set_current_dir(parent_dir.join(new_dir[1..].join("/"))).unwrap();
                                        set_current_dit(parent_dir, &new_dir[1..].join("/"));
                                    };
                                };
                            },
                            "." => {if new_dir[1..].len() > 0 && let Ok(parent_dir) = env::current_dir(){ 
                                        // env::set_current_dir(parent_dir.join(new_dir[1..].join("/"))).unwrap();
                                        set_current_dit(parent_dir.as_path(), &new_dir[1..].join("/"));
                                    }}
                            "" => if new_dir[1..].len() > 0 {println!("cd: {}: No such file or directory", &parts[1])},
                            _ => println!("cd: {}: No such file or directory", &parts[1]),
                        };
                    };
                }
}

fn type_functionality(parts : &Vec<String>){
        match get_command(&parts[1]) {
                Some(_) => println!("{} is a shell builtin", parts[1]),
                _ => if let Some(full_path) = is_executable_command(&parts[1]) {
                     println!("{} is {}", &parts[1], full_path.display())
                } else {
                    println!("{}: not found", &parts[1])
                }
}
}

fn not_shell_buitin(parts : &Vec<String>){
if let Some(_) = is_executable_command(&parts[0]) {
                     let status: Result<std::process::ExitStatus, io::Error> = Command::new(&parts[0])
                                    .args(&parts[1..])
                                    .status();
                    match status {
                        Ok(status) => {
                            if !status.success() {
                                println!("{}: command exited with status {}", parts[0], status);
                            }
                        },
                        Err(_) => println!("{}: command not found", parts[0]),
                    }
                } else {
                    println!("{}: command not found", parts[0])
                }
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();

        let parts: Vec<String> = parse_args(command.trim());
        if parts.is_empty() {
            println!("{}: command not found", command.trim());
            continue;
        }
        

        match get_command(&parts[0]) {
            Some(ShellBuiltins::ECHO) => println!("{}", &command[4..].trim()),
            Some(ShellBuiltins::EXIT) => break,
            Some(ShellBuiltins::PWD) => match env::current_dir() {
                Ok(path) => println!("{}", path.display()),
                Err(e) => println!("Error getting current directory: {}", e),
            },
            Some(ShellBuiltins::CD) => cd_functionality(&parts),
            Some(ShellBuiltins::TYPE) => type_functionality(&parts),
            _ => not_shell_buitin(&parts),
        }
    }
}