//! A command‑line todo list application that supports adding, removing, listing and marking tasks as done. The CLI is parsed with `clap` and the todo list is persisted to a JSON file using `serde_json`.

pub mod core {
    use serde::{Serialize, Deserialize};
    #[derive(Serialize, Deserialize)]
    pub struct TodoItem {
        pub id: u32,
        pub title: String,
        pub completed: bool,
    }

    impl TodoItem {
        /// Creates a new todo item with the given id and title.
        pub fn new(id: u32, title: String) -> Self {
            Self { id, title, completed: false }
        }

        /// Marks the todo item as completed.
        pub fn mark_done(&mut self) {
            self.completed = true;
        }
    }
}

pub mod cli {
    use std::env;

    /// Top‑level CLI configuration.
    pub struct Cli {
        pub command: Command,
    }

    /// All supported CLI commands.
    pub enum Command {
        Add { title: String },
        Remove { id: u32 },
        List,
        MarkDone { id: u32 },
    }

    /// Parses the current process arguments into a `Cli` instance.
    pub fn parse() -> Cli {
        // Skip the first argument (program name)
        let mut args = env::args().skip(1);

        match args.next() {
            Some(cmd) => match cmd.as_str() {
                "add" => {
                    // Join all remaining arguments as the title
                    let title = args.collect::<Vec<_>>().join(" ");
                    if title.is_empty() {
                        panic!("`add` command requires a title");
                    }
                    Cli {
                        command: Command::Add { title },
                    }
                }
                "remove" => {
                    let id_str = args
                        .next()
                        .expect("`remove` command requires an ID argument");
                    let id: u32 = id_str
                        .parse()
                        .expect("ID must be a valid unsigned integer");
                    Cli {
                        command: Command::Remove { id },
                    }
                }
                "list" => Cli {
                    command: Command::List,
                },
                "markdone" | "mark_done" => {
                    let id_str = args
                        .next()
                        .expect("`markdone` command requires an ID argument");
                    let id: u32 = id_str
                        .parse()
                        .expect("ID must be a valid unsigned integer");
                    Cli {
                        command: Command::MarkDone { id },
                    }
                }
                _ => panic!("Unknown command: `{}`", cmd),
            },
            None => {
                // Default to listing when no command is provided
                Cli {
                    command: Command::List,
                }
            }
        }
    }
}

pub mod errors {
    use std::io;
    use serde_json;

    /// Represents all possible errors that can occur in the application.
    #[derive(Debug)]
    pub enum AppError {
        Io(io::Error),
        Json(serde_json::Error),
        NotFound(u32),
    }

    /// Converts an `std::io::Error` into an `AppError`.
    impl From<io::Error> for AppError {
        fn from(err: io::Error) -> Self {
            AppError::Io(err)
        }
    }

    /// Converts a `serde_json::Error` into an `AppError`.
    impl From<serde_json::Error> for AppError {
        fn from(err: serde_json::Error) -> Self {
            AppError::Json(err)
        }
    }

    /// Implements `std::fmt::Display` for `AppError` to provide user‑friendly error messages.
    impl std::fmt::Display for AppError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                AppError::Io(err) => write!(f, "IO error: {}", err),
                AppError::Json(err) => write!(f, "JSON error: {}", err),
                AppError::NotFound(id) => write!(f, "Item with id {} not found", id),
            }
        }
    }
}

pub mod commands {
    use crate::core::{TodoItem};
    use crate::errors::AppError;

    /// Adds a new todo item to the list.
    pub fn add_todo(todos: &mut Vec<TodoItem>, title: String) -> Result<(), AppError> {
        // Determine the next id by taking the maximum existing id + 1
        let next_id = todos.iter().map(|t| t.id).max().unwrap_or(0) + 1;
        let item = TodoItem::new(next_id, title);
        todos.push(item);
        Ok(())
    }

    /// Removes a todo item by id.
    pub fn remove_todo(todos: &mut Vec<TodoItem>, id: u32) -> Result<(), AppError> {
        if let Some(pos) = todos.iter().position(|t| t.id == id) {
            todos.remove(pos);
            Ok(())
        } else {
            Err(AppError::NotFound(id))
        }
    }

    /// Returns a formatted string of all todo items.
    pub fn list_todos(todos: &[TodoItem]) -> String {
        let mut lines = Vec::new();
        for item in todos {
            let status = if item.completed { "[x]" } else { "[ ]" };
            lines.push(format!("{} {}: {}", status, item.id, item.title));
        }
        lines.join("\n")
    }

    /// Marks a todo item as completed.
    pub fn mark_done(todos: &mut Vec<TodoItem>, id: u32) -> Result<(), AppError> {
        if let Some(item) = todos.iter_mut().find(|t| t.id == id) {
            item.mark_done();
            Ok(())
        } else {
            Err(AppError::NotFound(id))
        }
    }
}

pub mod storage {
    use std::fs;
    
    use std::path::Path;

    use crate::core::TodoItem;
    use crate::errors::AppError;

    /// Loads the todo list from a JSON file.
    ///
    /// # Errors
    ///
    /// Returns an `AppError::Io` if the file cannot be opened,
    /// or an `AppError::Json` if the JSON is malformed.
    pub fn load_todos(path: &Path) -> Result<Vec<TodoItem>, AppError> {
        let file = fs::File::open(path).map_err(AppError::Io)?;
        serde_json::from_reader(file).map_err(AppError::Json)
    }

    /// Saves the todo list to a JSON file.
    ///
    /// # Errors
    ///
    /// Returns an `AppError::Io` if the file cannot be created/written,
    /// or an `AppError::Json` if serialization fails.
    pub fn save_todos(path: &Path, todos: &[TodoItem]) -> Result<(), AppError> {
        let file = fs::File::create(path).map_err(AppError::Io)?;
        serde_json::to_writer_pretty(file, todos).map_err(AppError::Json)
    }
}

pub mod main {
    use std::process;
    use std::path::Path;

    /// Entry point that wires together CLI parsing, command execution and persistence.
    fn main() {
        // Parse the command line arguments into a `Cli` struct
        let cli = crate::cli::parse();

        // Path to the JSON file that stores the todo list
        let path = Path::new("todos.json");

        // Load existing todos from disk (or start with an empty list on error)
        let mut todos = match crate::storage::load_todos(path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Error loading todos: {}", e);
                process::exit(1);
            }
        };

        // Execute the requested command
        let cmd_result = match cli.command {
            crate::cli::Command::Add { title } => {
                crate::commands::add_todo(&mut todos, title)
            }
            crate::cli::Command::Remove { id } => {
                crate::commands::remove_todo(&mut todos, id)
            }
            crate::cli::Command::MarkDone { id } => {
                crate::commands::mark_done(&mut todos, id)
            }
            crate::cli::Command::List => {
                let output = crate::commands::list_todos(&todos);
                println!("{}", output);
                Ok(())
            }
        };

        // If the command failed, report and exit
        if let Err(e) = cmd_result {
            eprintln!("Error: {}", e);
            process::exit(1);
        }

        // Persist the updated todo list back to disk
        if let Err(e) = crate::storage::save_todos(path, &todos) {
            eprintln!("Error saving todos: {}", e);
            process::exit(1);
        }
    }
}

fn main() {
    println!("Starting application...");
    todo!("Wire up application entry point")
}