use std::{fs, path::PathBuf};

use dialoguer::{console::style, theme::Theme, Input};

use super::Wizard;

impl<T: Theme> Wizard<T> {
    pub fn get_whitelist_path(&self) -> PathBuf {
        loop {
            let path: String = Input::with_theme(&self.theme)
                .with_prompt("Enter whitelist file location")
                .interact_text()
                .unwrap();
            let path = PathBuf::from(path);

            let metadata = match fs::metadata(&path) {
                Ok(metadata) => metadata,
                Err(e) => {
                    eprintln!("{} {}", style("Failed to inspect file:").red().bold(), e);

                    continue;
                }
            };

            if metadata.is_dir() {
                eprintln!("{}", style("Specified path is a directory").red().bold());

                continue;
            }

            break path;
        }
    }
}
