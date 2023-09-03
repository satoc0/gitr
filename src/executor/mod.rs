use std::collections::HashMap;
use std::process::Command;
use std::sync::mpsc;
use std::{thread, env};

static GIT_HISTORY_FILE_NAME: &str = ".gitt-history.txt";

use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result as RLResult};
use spinoff::{Spinner, spinners, Color};
use colored::*;

const VALID_SYMBOL: char = '✔';
const ERROR_SYMBOL: char = '✖';

pub const ALL_GIT_COMMANDS: [&str; 29] = [
    "init",
    "clone",
    "add",
    "mv",
    "reset",
    "rm",
    "bisect",
    "grep",
    "log",
    "show",
    "status",
    "branch",
    "checkout",
    "commit",
    "diff",
    "merge",
    "rebase",
    "tag",
    "fetch",
    "pull",
    "push",
    "remote",
    "shortlog",
    "stash",
    "config",
    "help",
    "config",
    "am",
    "cherry-pick",
];

use crate::config::{ConfigManager, GitDirectoryInfo, Config};

struct CommandStatus {
    symbol: char,
    output: String
}

pub struct Runtime {
    config: ConfigManager,
    current_cmd_status: HashMap<String, CommandStatus>,
}

impl Runtime {
    pub fn create(config: ConfigManager) -> Runtime {
        let current_branch: &Vec<GitDirectoryInfo> = &config.get_config().workable_paths;

        let status_hash_map: HashMap<String, CommandStatus> = current_branch
            .iter()
            .map(|git_dic| {
                let status = CommandStatus {
                    symbol: '⌛',
                    output: String::new(),
                };
                (git_dic.name.to_string(), status)
            })
            .collect();

        let runtime = Runtime { config, current_cmd_status: status_hash_map };

        return runtime;
    }

    pub fn start(&mut self) -> RLResult<()> {
        let mut rl: rustyline::Editor<(), rustyline::history::FileHistory> = DefaultEditor::new()?;

        if rl.load_history(GIT_HISTORY_FILE_NAME).is_err() {
            println!("No previous history.");
        }

        loop {
            let read_line_prompt = self.get_promp();

            let readline: Result<String, ReadlineError> = rl.readline(read_line_prompt.as_str());
            match readline {
                Ok(line) => {
                    let line_str = line.as_str();
                    rl.add_history_entry(line_str).unwrap();

                    self.parse_line(line_str);

                    rl.save_history(GIT_HISTORY_FILE_NAME).unwrap();
                }
                Err(ReadlineError::Interrupted) => {
                    println!("Exit");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("Done");
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }

        Ok(())
    }

    fn get_promp(&self) -> String {

        let has_some_error: bool = self.has_some_error_on_last_cmd();
        let prompt_header = self.get_prompt_header(has_some_error);
        let prefix_signal: ColoredString = if has_some_error { "$".red() } else { "$".bright_blue()};

        let current_dir_path_buff = env::current_dir().expect("Error");

        let mut dir_name: String = String::from(current_dir_path_buff.file_name().expect("").to_string_lossy().into_owned());

        dir_name.insert(0, ' ');
        dir_name.push(' ');

        // String::from_utf8_lossy(&current_dir_path_buff).to_string();
        let read_line_prompt: String = format!("{} {}\n{} {} ", dir_name.reversed(), prompt_header, prefix_signal, "git".bright_white());

        read_line_prompt
    }

    fn get_prompt_header(&self, has_some_error: bool) -> String {
        let config: &Config = self.config.get_config();
        let branchs: &Vec<GitDirectoryInfo> = &config.workable_paths;
        let first_git_dic = &branchs[0].path;
        
        let output = Command::new("git")
            .current_dir(&first_git_dic)
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output().expect("Error");

        let output_str_vec = if output.status.success() { output.stdout } else { output.stderr };
        let current_branch = String::from_utf8_lossy(&output_str_vec).to_string();

        let mut prompt_str: String = String::from("");

        let mut branch_name_styled: String = String::from("");
        if has_some_error {
            branch_name_styled.push_str("◖".bright_red().to_string().as_str());
            branch_name_styled.push_str(format!("branch: {}", current_branch.trim().bright_red()).as_str());
            branch_name_styled.push_str("◗".bright_red().to_string().as_str());
        } else {
            branch_name_styled.push_str("◖".bright_blue().to_string().as_str());
            branch_name_styled.push_str(format!("branch: {}", current_branch.trim().bright_blue()).as_str());
            branch_name_styled.push_str("◗".bright_blue().to_string().as_str());
        }

        prompt_str.push_str(branch_name_styled.as_str());

        let branches: Vec<(&String, String)> = branchs.iter().map( |git_dic| (&git_dic.name, get_branch_name(&git_dic.path))).collect();

        let is_sync: bool = branches.iter().all(|a| a.1.to_string() == branches[0].1.to_string());

        if is_sync {
            prompt_str.push_str(" SYNCED ".bright_green().to_string().as_str());
        } else {
            prompt_str.push_str(" NOT SYNCED ".bright_red().to_string().as_str());
            let all_branchs_string: Vec<String> = branches.iter().map( |b| {
                let mut str = String::from(b.0.trim());
                str.push(':');
                str.push_str(b.1.trim().to_string().as_str());
                str
            }).collect();

            prompt_str.push_str(all_branchs_string.join(", ").as_str());
        }

        prompt_str
    }

    fn parse_line(&mut self, line: &str) {
        if line.len() == 0 {
            println!("Type some command");
            return
        }

        let args: Vec<&str> = line.split_whitespace().collect();
        let command: &str = args[0];

        if command == "clear" {
            println!("\x1B[2J\x1B[1;1H");
            return
        }

        if command == "branches" {
            let config: &Config = self.config.get_config();
            let branchs: &Vec<GitDirectoryInfo> = &config.workable_paths;
            match ConfigManager::init_first_config(branchs.clone()) {
                Ok(config) => {
                    self.config.config = config;
                }
                Err(err) => {
                    println!("{}", err);
                }
            }
            return
        }

        if !ALL_GIT_COMMANDS.contains(&command) {
            println!("Type a valid git command. \"{}\" doesn't exist.", command);
        } else {
            let config: &Config = self.config.get_config();
            let branchs: &Vec<GitDirectoryInfo> = &config.workable_paths;

            self.current_cmd_status = branchs
                .iter()
                .map(|git_dic| {
                    let status = CommandStatus {
                        symbol: '⌛',
                        output: String::new(),
                    };
                    (git_dic.name.to_string(), status)
                })
                .collect();

            let mut spinner = Spinner::new(spinners::Dots, "", Color::Blue); 
            
            let mut started_threads = 0;
            let (tx, rx) = mpsc::channel();

            for git_dic in branchs.iter() {
                let tx = tx.clone();

                started_threads += 1;

                let args_clone: Vec<String> = args.clone().into_iter().map(|e| e.to_string()).collect();
                let git_dic_clone: GitDirectoryInfo = git_dic.clone();
                let git_dic_name: String = git_dic_clone.name.clone();
                
                thread::spawn(move || {
                    let result = Self::execute_command(&git_dic_clone, args_clone);
                    tx.send((git_dic_clone, result)).expect(format!("Error send result to parent thread: {}", git_dic_name).as_str())
                });
            }

            let mut completed_threads: i32 = 0;
            let mut some_error: bool = false;

            while completed_threads < started_threads {
                if let Ok((git_dic, result)) = rx.recv() {
                    completed_threads += 1;
                    match result {
                        Ok(output) => {
                            self.current_cmd_status.insert(git_dic.name.to_string(), get_valid_cmd_status(output));
                        }
                        Err(error) => {
                            some_error = true;
                            self.current_cmd_status.insert(git_dic.name.to_string(), get_error_cmd_status(error));
                        }
                    }
                }
            }

            spinner.update_text(self.get_cmd_results());
            spinner.stop();

            if some_error {
                println!("\n{}", self.get_cmd_errors_summary())
            }
        }
    }

    fn get_cmd_results(&self) -> String {
        let mut status_keys: Vec<&String> = self.current_cmd_status.keys().collect();
        status_keys.sort();

        let mut result = String::new();
        for key in status_keys {
            if let Some(status) = self.current_cmd_status.get(key) {
                let symbol_string: ColoredString = if status.symbol == VALID_SYMBOL {
                    status.symbol.to_string().bright_green()
                } else {
                    status.symbol.to_string().red()
                };

                result.push_str(&format!("{} {} | ", symbol_string, key));
            }
        }

        result[0..result.len() - 3].to_string()
    }

    fn has_some_error_on_last_cmd(&self) -> bool {
        let status_keys: Vec<&CommandStatus> = self.current_cmd_status.values().collect();

        return status_keys.iter().any(|v| v.symbol == ERROR_SYMBOL);
    }

    fn get_cmd_errors_summary(&self) -> String {
        self.current_cmd_status.iter()
            .filter(|(_, status)| status.symbol == ERROR_SYMBOL)
            .map(|(key, status)| format!("{} {}\n{}\n", status.symbol.to_string().bright_red(), key, status.output))
            .collect::<String>()
    }

    fn execute_command(git_dic: &GitDirectoryInfo, args: Vec<String>) -> Result<String, String> {
        let exec = Command::new("git")
            .current_dir(&git_dic.path)
            .args(args)
            .output();

        match exec {
            Ok(output) => {
                let output_str_vec = if output.status.success() { output.stdout } else { output.stderr };
                let output_string = String::from_utf8_lossy(&output_str_vec).to_string();

                if output.status.success() {
                    return Ok(output_string)
                } else {
                    return Err(output_string)
                };
            },
            Err(err) => {
                return Err(err.to_string())
            },
        }
        
    }
}

fn get_valid_cmd_status(output: String) -> CommandStatus {
    CommandStatus {
        symbol: VALID_SYMBOL,
        output,
    }
}

fn get_error_cmd_status(output: String) -> CommandStatus {
    CommandStatus {
        symbol: ERROR_SYMBOL,
        output,
    }
}

fn get_branch_name(path: &String) -> String {
    let output = Command::new("git")
        .current_dir(path)
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output().expect("Error");

    let output_str_vec = if output.status.success() { output.stdout } else { output.stderr };
    String::from_utf8_lossy(&output_str_vec).to_string()
}