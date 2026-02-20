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
use anyhow::Result;
use std::fs::File;
#[allow(unused_imports)]
use std::io::{self, Write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::{env, path::PathBuf};

enum ShellBuiltins {
    ECHO,
    EXIT,
    TYPE,
    PWD,
    CD,
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

fn execute_with_redirection(cmd: &str, args: &[String], file_name: &str) -> Result<()> {
    // Create the file
    let file = File::create(file_name)?;

    Command::new(cmd)
        .args(args)
        .stdout(Stdio::from(file)) // OS pipes output directly to disk
        .status()?; // The ? automatically converts io::Error into anyhow::Error if it fails

    return Ok(());
}

fn is_executable_command(command: &str) -> Option<PathBuf> {
    if let Some(path_env) = env::var_os("PATH") {
        let exe_array: [&str; 4] = ["", "exe", "bat", "cmd"];
        for dir in env::split_paths(&path_env) {
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
                        } else {
                            return false;
                        }
                    } else {
                        return false;
                    }
                } else {
                    full_path.with_extension(ext).exists()
                }
            }) {
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

    let mut iter: std::iter::Peekable<std::str::Chars<'_>> = input.chars().peekable();

    while let Some(c) = iter.next() {
        match (quote_char, c) {
            (Some(q), c) if q == c => {
                quote_char = None;
            }
            (None, '\'' | '"') => {
                quote_char = Some(c);
            }
            (None, ' ') | (None, '\t') => {
                if !current_arg.is_empty() {
                    args.push(current_arg.clone());
                    current_arg.clear();
                }
            }
            (None | Some('"'), '\\') => {
                if let Some(next_char) = iter.next() {
                    current_arg.push(next_char);
                }
            }
            // (_, '>') => {
            //     execute_with_redirection(
            //         &args[0],
            //         &args[1..],
            //         &input[input.find('>').unwrap() + 1..].trim(),
            //     )
            //     .unwrap();
            //     return [].to_vec();
            // }
            (_, c) => {
                current_arg.push(c);
            }
        }
    }
    if !current_arg.is_empty() {
        args.push(current_arg);
    }

    args
}

fn set_current_dit(parent_path: &Path, path: &str) {
    let new_path: PathBuf = parent_path.join(path);
    if new_path.exists() {
        env::set_current_dir(new_path).unwrap();
    } else {
        println!("cd: {}: No such file or directory", &path);
    }
}

fn cd_functionality(parts: &Vec<String>) {
    if parts.len() < 2 {
    } else if parts.len() > 2 {
        println!("too many arguments");
    } else {
        if parts[1].starts_with("/") {
            let new_path = Path::new(&parts[1]);
            if new_path.is_absolute() && new_path.exists() {
                env::set_current_dir(&parts[1]).unwrap();
                return;
            } else {
                println!("cd: {}: No such file or directory", &parts[1]);
                return;
            }
        } else {
            let new_dir: Vec<&str> = parts[1].split("/").collect();
            match new_dir[0] {
                "~" => env::set_current_dir(env::home_dir().unwrap()).unwrap(),
                ".." => {
                    if let Some(parent_dir) = env::current_dir().unwrap().parent() {
                        env::set_current_dir(parent_dir).unwrap();
                        if new_dir[1..].len() > 0 {
                            // env::set_current_dir(parent_dir.join(new_dir[1..].join("/"))).unwrap();
                            set_current_dit(parent_dir, &new_dir[1..].join("/"));
                        };
                    };
                }
                "." => {
                    if new_dir[1..].len() > 0
                        && let Ok(parent_dir) = env::current_dir()
                    {
                        // env::set_current_dir(parent_dir.join(new_dir[1..].join("/"))).unwrap();
                        set_current_dit(parent_dir.as_path(), &new_dir[1..].join("/"));
                    }
                }
                "" => {
                    if new_dir[1..].len() > 0 {
                        println!("cd: {}: No such file or directory", &parts[1])
                    }
                }
                _ => println!("cd: {}: No such file or directory", &parts[1]),
            };
        };
    }
}

fn pwd_functionality(stream: &mut dyn Write) {
    match env::current_dir() {
        Ok(path) => writeln!(stream, "{}", path.display()).unwrap(),
        Err(e) => writeln!(stream, "Error getting current directory: {}", e).unwrap(),
    }
}

fn type_functionality(parts: &Vec<String>, stream: &mut dyn Write) {
    match get_command(&parts[1]) {
        Some(_) => writeln!(stream, "{} is a shell builtin", parts[1]).unwrap(),
        _ => {
            if let Some(full_path) = is_executable_command(&parts[1]) {
                writeln!(stream, "{} is {}", &parts[1], full_path.display()).unwrap();
            } else {
                writeln!(stream, "{}: not found", &parts[1]).unwrap();
            }
        }
    }
}

fn not_shell_buitin(parts: &Vec<String>, redirect_file: &Option<String>) {
    match redirect_file {
        Some(file) => {
            if let Err(_) = execute_with_redirection(&parts[0], &parts[1..], file) {
                println!("{}: command not found", parts[0]);
            }
        }
        None => {
            if let Some(_) = is_executable_command(&parts[0]) {
                let status: Result<std::process::ExitStatus, io::Error> =
                    Command::new(&parts[0]).args(&parts[1..]).status();
                match status {
                    Ok(status) => {
                        if !status.success() {
                            println!("{}: command exited with status {}", parts[0], status);
                        }
                    }
                    Err(_) => println!("{}: command not found", parts[0]),
                }
            } else {
                println!("{}: command not found", parts[0])
            }
        }
    }
}

fn echo_functionality(parts: &[String], stream: &mut dyn Write) {
    writeln!(stream, "{}", parts.join(" ")).unwrap();
}

fn create_stream(redirect_file: &Option<String>) -> Box<dyn Write> {
    let stream: Box<dyn Write> = match &redirect_file {
        Some(file) => Box::new(File::create(file).unwrap()),
        None => Box::new(io::stdout()),
    };
    return stream;
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();

        let mut parts: Vec<String> = parse_args(command.trim());
        if parts.is_empty() {
            // println!("{}: command not found", command.trim());
            continue;
        }

        let mut redirect_file: Option<String> = None;
        if let Some(pos) = parts.iter().position(|p| p == ">") {
            if pos + 1 < parts.len() {
                redirect_file = Some(parts[pos + 1].clone());
                parts.truncate(pos); // Clean the parts array!
            }
        }
        let mut stream = create_stream(&redirect_file);

        match get_command(&parts[0]) {
            Some(ShellBuiltins::ECHO) => echo_functionality(&parts[1..], &mut *stream),
            Some(ShellBuiltins::EXIT) => break,
            Some(ShellBuiltins::PWD) => pwd_functionality(&mut *stream),
            Some(ShellBuiltins::CD) => cd_functionality(&parts),
            Some(ShellBuiltins::TYPE) => type_functionality(&parts, &mut *stream),
            _ => not_shell_buitin(&parts, &redirect_file),
        }
    }
}
