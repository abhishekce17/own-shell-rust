use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    queue,
    style::Print,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
};
#[allow(unused_imports)]
use std::io::{self, Write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::{
    cmp::min,
    fs::{File, OpenOptions},
};
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

#[cfg(windows)]
const EXE_ARRAY: &[&str] = &["exe", "bat", "cmd", "com"];

#[cfg(unix)]
const EXE_ARRAY: &[&str] = &[""];

impl ShellBuiltins {
    const ALL_STRINGS: [&'static str; 5] = ["echo", "exit", "type", "pwd", "cd"];
}

fn read_input_with_autocomplete() -> Result<String> {
    // 1. Enter raw mode just for typing
    enable_raw_mode()?;
    let mut stdout = io::stdout();

    let mut input = String::new();
    let mut cursor_pos = 0; // Track exactly where the blinking cursor should be
    let mut tab_pressed_count: i32 = 0;

    loop {
        // 2. FLICKER-FREE RENDER LOOP
        // We queue all drawing commands in memory first to prevent screen tearing
        queue!(
            stdout,
            cursor::MoveToColumn(0),                       // Move to far left
            Clear(ClearType::CurrentLine),                 // Erase the old text
            Print(format!("$ {}", input)),                 // Draw the prompt and text
            cursor::MoveToColumn((cursor_pos + 2) as u16) // Move cursor back to the editing position (+2 for "$ ")
        )?;
        // Send the entire frame to the monitor instantly
        stdout.flush()?;

        // 3. WAIT FOR KEYPRESS
        if let Event::Key(key) = event::read()? {
            // Ignore key releases (Windows compatibility)
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match key.code {
                // --- SUBMITTING ---
                KeyCode::Enter => {
                    print!("\r\n");
                    break;
                }

                // --- ARROW KEYS (NAVIGATION) ---
                KeyCode::Left => {
                    if cursor_pos > 0 {
                        cursor_pos -= 1;
                    }
                }
                KeyCode::Right => {
                    if cursor_pos < input.len() {
                        cursor_pos += 1;
                    }
                }

                // --- EDITING ---
                KeyCode::Backspace => {
                    if cursor_pos > 0 {
                        input.remove(cursor_pos - 1);
                        cursor_pos -= 1;
                    }
                }

                KeyCode::Char(c) => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        match c {
                            'c' => {
                                print!("^C\r\n");
                                input.clear();
                                break;
                            }
                            'd' => {
                                disable_raw_mode()?;
                                std::process::exit(0);
                            }
                            // The CodeCrafters Fix: Treat Ctrl+J (\n) and Ctrl+M (\r) as Enter
                            'j' | 'm' => {
                                print!("\r\n");
                                break;
                            }
                            _ => {} // Ignore other weird control chars
                        }
                    } else {
                        // Insert the character EXACTLY where the cursor is
                        input.insert(cursor_pos, c);
                        cursor_pos += 1;
                    }
                }

                // --- AUTOCOMPLETION ---
                KeyCode::Tab => {
                    tab_pressed_count += 1;
                    stdout.flush()?;
                    // 1. Try to match Builtins first
                    let builtin_match = ShellBuiltins::ALL_STRINGS
                        .iter()
                        .find(|&&cmd| cmd.starts_with(&input));

                    if let Some(matched) = builtin_match {
                        input = matched.to_string();
                        input.push(' ');
                        cursor_pos = input.len();
                    } else {
                        match find_all_match_in_path(&input) {
                            Some(matches) => {
                                if matches.len() == 1 {
                                    input = matches[0].clone();
                                    input.push(' ');
                                    cursor_pos = input.len();
                                    tab_pressed_count = 0;
                                } else if matches.len() > 1
                                    && let common_prefix = longest_common_prefix(&matches)
                                    && common_prefix.len() > input.len()
                                {
                                    input = common_prefix;
                                    cursor_pos = input.len();
                                    tab_pressed_count = 0;
                                } else if matches.len() > 1 && tab_pressed_count == 2 {
                                    println!("\r\n{}", matches.join("  "));
                                    // Re-render the prompt and current input after showing options
                                    queue!(
                                        stdout,
                                        cursor::MoveToColumn(0),
                                        Clear(ClearType::CurrentLine),
                                        Print(format!("$ {}", input)),
                                        cursor::MoveToColumn((cursor_pos + 2) as u16)
                                    )?;
                                    stdout.flush()?;
                                    tab_pressed_count = 0; // Reset the count after showing options
                                } else {
                                    print!("\x07");
                                    io::stdout().flush()?;
                                }
                            }
                            None => {
                                print!("\x07");
                                io::stdout().flush()?;
                            }
                        }
                    }
                }

                _ => {} // Ignore any other keys
            }
        }
    }

    disable_raw_mode()?;
    Ok(input)
}

fn find_all_match_in_path(partial_input: &str) -> Option<Vec<String>> {
    if partial_input.is_empty() {
        return None;
    }
    if let Some(path_env) = env::var_os("PATH") {
        let mut matches: Vec<String> = Vec::new();
        for dir in env::split_paths(&path_env) {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.starts_with(partial_input)
                            && EXE_ARRAY.iter().any(|&ext| name.ends_with(ext))
                            && is_executable_path(&entry.path())
                        {
                            matches.push(name.to_string());
                        }
                    }
                }
            }
        }
        if !matches.is_empty() {
            return Some(matches);
        }
    }
    return None;
}

fn longest_common_prefix(matches: &Vec<String>) -> String {
    if matches.is_empty() {
        return "".to_string();
    }
    let mut strings = matches.clone();
    strings.sort();
    let first_match: Vec<char> = strings[0].chars().collect();
    let last_match: Vec<char> = strings[strings.len() - 1].chars().collect();
    let mut lcp: Vec<char> = Vec::new();
    for i in 0..min(first_match.len(), last_match.len()) {
        if first_match[i] != last_match[i] {
            return lcp.into_iter().collect();
        }
        lcp.push(first_match[i]);
    }
    return lcp.into_iter().collect();
}

fn execute_with_redirection(
    cmd: &str,
    args: &[String],
    file_name: &str,
    redirect_err: bool,
    is_append: bool,
) -> Result<()> {
    // Create the file
    let file = File::options()
        .create(true)
        .write(true)
        .append(is_append)
        .truncate(!is_append)
        .open(file_name)?;

    let mut cmd: Command = Command::new(cmd); // The ? automatically converts io::Error into anyhow::Error if it fails
    cmd.args(args);
    if redirect_err {
        cmd.stderr(Stdio::from(file));
    } else {
        cmd.stdout(Stdio::from(file));
    }

    cmd.status()?;
    return Ok(());
}

fn is_executable_path(full_path: &PathBuf) -> bool {
    if full_path.is_file() {
        #[cfg(unix)]
        {
            if let Ok(metadata) = full_path.metadata() {
                if metadata.permissions().mode() & 0o111 != 0 {
                    return true;
                }
            }
        }

        #[cfg(windows)]
        {
            return true;
        }
    }
    EXE_ARRAY.iter().any(|&ext| {
        if ext.is_empty() {
            return false;
        }
        full_path.with_extension(ext).is_file()
    })
}

fn is_variable_path(command: &str) -> Option<PathBuf> {
    if let Some(path_env) = env::var_os("PATH") {
        for dir in env::split_paths(&path_env) {
            let full_path = dir.join(command);
            if is_executable_path(&full_path) {
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
            if let Some(full_path) = is_variable_path(&parts[1]) {
                writeln!(stream, "{} is {}", &parts[1], full_path.display()).unwrap();
            } else {
                writeln!(stream, "{}: not found", &parts[1]).unwrap();
            }
        }
    }
}

fn not_shell_buitin(
    parts: &Vec<String>,
    redirect_file: &Option<String>,
    redirect_err: bool,
    is_append: bool,
) {
    if let Some(file) = redirect_file {
        // This is where the magic happens
        if let Err(e) =
            execute_with_redirection(&parts[0], &parts[1..], file, redirect_err, is_append)
        {
            eprintln!("Error executing: {}", e);
        }
    } else {
        // Standard non-redirected logic...
        if let Some(_) = is_variable_path(&parts[0]) {
            let _ = Command::new(&parts[0]).args(&parts[1..]).status();
        } else {
            println!("{}: command not found", parts[0]);
        }
    }
}

fn echo_functionality(parts: &[String], stream: &mut dyn Write) {
    writeln!(stream, "{}", parts.join(" ")).unwrap();
}

fn create_stream(
    redirect_file: &Option<String>,
    redirect_err: bool,
    is_append: bool,
) -> Box<dyn Write> {
    if let Some(file_path) = redirect_file {
        let file = OpenOptions::new()
            .create(true) // Create it if it doesn't exist
            .write(true) // We need write permission
            .append(is_append) // IF TRUE: Seek to end before every write
            .truncate(!is_append) // IF TRUE: Wipe the file clean (standard >)
            .open(file_path)
            .unwrap();
        if !redirect_err {
            return Box::new(file);
        }
    }
    Box::new(io::stdout())
}

fn main() {
    loop {
        // print!("$ ");
        // io::stdout().flush().unwrap();
        // let mut command = String::new();
        // io::stdin().read_line(&mut command).unwrap();

        let command = match read_input_with_autocomplete() {
            Ok(cmd) => cmd,
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        };

        let mut parts: Vec<String> = parse_args(command.trim());
        if parts.is_empty() {
            // println!("{}: command not found", command.trim());
            continue;
        }

        let mut redirect_file: Option<String> = None;
        let mut redirect_err: bool = false;
        let mut is_append: bool = false;
        let pos: Option<usize> = parts.iter().position(|p| {
            p == ">" || p == "1>" || p == "2>" || p == ">>" || p == "1>>" || p == "2>>"
        });
        if let Some(pos) = pos {
            let token = parts[pos].as_str();
            if token.starts_with('2') {
                redirect_err = true;
            }
            if token.contains(">>") {
                is_append = true;
            }
            if pos + 1 < parts.len() {
                redirect_file = Some(parts[pos + 1].clone());
                parts.truncate(pos);
            }
        }

        match get_command(&parts[0]) {
            Some(builtin) => {
                let mut stream: Box<dyn Write> =
                    create_stream(&redirect_file, redirect_err, is_append);
                match builtin {
                    ShellBuiltins::ECHO => echo_functionality(&parts[1..], &mut *stream),
                    ShellBuiltins::EXIT => break,
                    ShellBuiltins::PWD => pwd_functionality(&mut *stream),
                    ShellBuiltins::CD => cd_functionality(&parts),
                    ShellBuiltins::TYPE => type_functionality(&parts, &mut *stream),
                }
            }
            _ => not_shell_buitin(&parts, &redirect_file, redirect_err, is_append),
        }
    }
}
