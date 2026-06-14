use std::path::PathBuf;

pub struct ScrollReadError {
    pub msg: String,
}

pub struct ScrollReader {
}

impl ScrollReader {
    pub fn new() -> Self {
        ScrollReader {
        }
    }

    pub fn read_scroll(&self, path: &PathBuf) -> Result<String, ScrollReadError> {
        let f = std::fs::read_to_string(path);

        if f.is_err() {
            println!(
                "The eira was cursed while reading the scroll '{}'.",
                path.display()
            );
            let msg: String;
            match &f.err().unwrap().kind() {
                std::io::ErrorKind::NotFound => {
                    msg = format!(
                        "The scroll '{}' could not be found. Has it been lost to the void?",
                        path.display()
                    );
                }
                std::io::ErrorKind::PermissionDenied => {
                    msg = format!(
                        "The scroll '{}' is protected by ancient magic. You don't have permission to read it.",
                        path.display()
                    );
                }
                std::io::ErrorKind::IsADirectory => {
                    msg =
                        "The path you gave is not of a scroll, but an archive (directory).".into();
                }
                std::io::ErrorKind::InvalidData => {
                    msg = format!(
                        "The scroll '{}' is corrupted and cannot be read.",
                        path.display()
                    );
                }
                _ => {
                    msg = format!(
                        "The eira was struck by unknown curses while reading the scroll '{}'.",
                        path.display()
                    );
                }
            };

            return Err(ScrollReadError {
                msg: msg.to_string(),
            });
        };

        Ok(f.ok().unwrap())
    }
}
