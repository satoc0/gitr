use std::fs::{self, File};
use std::io::Write;

use inquire::{list_option::ListOption, validator::Validation, MultiSelect};
use serde::{Deserialize, Serialize};

static CONFIG_FILE_NAME: &str = ".gitt-config.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GitDirectoryInfo {
    pub name: String,
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub workable_paths: Vec<GitDirectoryInfo>,
}

#[derive(Debug)]
pub struct ConfigManager<> {
    pub config: Config
}

impl ConfigManager<> {
    pub fn start() -> Result<ConfigManager, &'static str> {
        let mut instance = ConfigManager {
            config: Self::get_config_from_disk(),
        };

        if instance.config.workable_paths.clone().is_empty() {
            match Self::init_first_config(instance.config.workable_paths.clone()) {
                Ok(config) => {
                    instance.config = config;
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }

        return Ok(instance)
    }

    pub fn get_config(&self) -> &Config {
        return &self.config;
    }

    fn get_config_from_disk() -> Config {
        let mut config: Config = Config {
            workable_paths: vec![],
        };

        if let Ok(_) = fs::metadata(CONFIG_FILE_NAME) {
            let contents: String =
                fs::read_to_string(CONFIG_FILE_NAME).expect("Fail to read the config file");

            config = serde_json::from_str(&contents).unwrap();
        } else {
            Self::save_config_to_disk(&config);
        }

        config
    }

    fn save_config_to_disk(config: &Config) {
        let mut file: File = File::create(CONFIG_FILE_NAME).unwrap();

        let config_str: String = serde_json::to_string(config).unwrap();

        file.write_all(config_str.as_bytes()).unwrap();
    }

    pub fn init_first_config(workable_paths: Vec<GitDirectoryInfo>) -> Result<Config, &'static str> {
        let mut config = Config {
            workable_paths: vec![],
        };

        let git_directories_search: Vec<GitDirectoryInfo> = Self::get_git_directories_path();

        if git_directories_search.is_empty() {
            return Err("No git repository found")
        }

        let options: Vec<String> = git_directories_search
            .iter()
            .map(|d| d.name.to_string())
            .collect();

        let validator = |a: &[ListOption<&String>]| {
            if a.is_empty() {
                return Ok(Validation::Invalid("Select at least 1 repository!".into()));
            }

            Ok(Validation::Valid)
        };

        let my_slice: Vec<usize> = workable_paths.iter().map( |a| {
            let index = options.iter().position(|opt| opt.to_string() == a.name);

            index.unwrap()
        }).collect();

        let quest =
            MultiSelect::new("Select the repositories that you want include:", options)
                .with_validator(validator)
                .with_default(my_slice.as_slice());
                
        let ans: Result<Vec<String>, inquire::InquireError> = quest.prompt();

        match ans {
            Ok(answer) => {
                for branch_name in answer {
                    if let Some(path) = git_directories_search
                        .iter()
                        .find(|option: &&GitDirectoryInfo| option.name == branch_name)
                    {
                        config.workable_paths.push(GitDirectoryInfo {
                            name: path.name.to_string(),
                            path: path.path.to_string(),
                        });
                    }
                }

                Self::save_config_to_disk(&config);

                return Ok(config);
            }
            Err(_) => {
                Err("Repository list could not be processed")
            }
        }
    }

    fn get_git_directories_path() -> Vec<GitDirectoryInfo> {
        let mut response: Vec<GitDirectoryInfo> = vec![];

        let curr_dir: fs::ReadDir = fs::read_dir("./").unwrap();

        for dir in curr_dir {
            let d: fs::DirEntry = dir.unwrap();

            let is_file: bool = d.metadata().unwrap().is_file();

            if is_file {
                continue;
            }

            let dir_name: String = d.file_name().into_string().unwrap();
            let dir_path: String = d.path().into_os_string().into_string().unwrap();

            let mut read_dir: fs::ReadDir = fs::read_dir(&dir_path).unwrap();

            let is_git_directory: bool = read_dir.any(|d| d.unwrap().file_name() == ".git");

            if !is_git_directory {
                continue;
            }

            let git_directory_info: GitDirectoryInfo = GitDirectoryInfo {
                name: dir_name,
                path: dir_path,
            };

            response.push(git_directory_info)
        }

        return response;
    }
}
