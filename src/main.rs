//! A command‑line todo list application that supports adding, removing, listing and marking tasks as done. The CLI is parsed with `clap` and the todo list is persisted to a JSON file using `serde_json`.

pub mod core {
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
    //! Parses command‑line arguments into a strongly typed command structure.
    todo!()
}

pub mod errors {
    use std::io;
    use serde_json;

    /// Represents all possible errors that can occur in the application.
    #[derive(Debug)]
    pub enum AppError {
        /// I/O related error
        Io(io::Error),
        /// JSON parsing / serialization error
        Json(serde_json::Error),
        /// Entity with the given id was not found
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
}

pub mod commands {
    //! Implements the business logic for each CLI command.
    todo!()
}

pub mod storage {
    //! Handles reading from and writing to the JSON persistence file.
    todo!()
}

pub mod main {
    //! Entry point that wires together CLI parsing, command execution and persistence.
    todo!()
}

fn main() {
    println!("Starting application...");
    todo!("Wire up application entry point")
}
