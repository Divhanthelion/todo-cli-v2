# 🔥 BALLIN' TODO LIST - The Ultimate CLI Task Manager

A hyper-charged command-line todo list application written in Rust. It takes the simplicity of a CLI todo list and completely supes it up with colors, tags, statistics, and a full undo/redo stack.

## Features

- **🌈 Color-Coded Interface:** Beautiful terminal UI with colors indicating status, priority, and categories.
- **📅 Due Dates & Deadlines:** Full date parsing with overdue warnings and "due-soon" filters.
- **🏷️ Tagging System:** Organize your life with comma-separated tags and contextual categories.
- **📊 Progress Tracking:** Run `todo stats` to see a dashboard of your completion rate and task breakdown.
- **🔍 Advanced Search & Filtering:** Filter by `done`, `undone`, `overdue`, or search directly by string/tag.
- **↩️ Robust State Management:** Full Undo/Redo support for preventing accidental deletions or changes.

## Installation

Ensure you have Rust installed, then build from source:

```bash
git clone https://github.com/your-username/todo-cli-v2.git
cd todo-cli-v2
cargo install --path .
```

## Usage

```bash
todo <command> [options]
```

### Commands

- **`add`**: Add a new task
  - Usage: `todo add "Task Title" [-p priority] [-d date] [-t tags] [-c category]`
  - Example: `todo add "Buy groceries" -p high -d 2024-12-31 -t food,shopping`
- **`list`**: View tasks
  - Usage: `todo list [filter] [--sort FIELD]`
  - Filters: `done`, `undone`, `overdue`, `due-soon`
  - Sorts: `priority`, `date`, `id`
- **`done <id>`**: Mark a task as complete.
- **`undo-task <id>`**: Mark a completed task as incomplete.
- **`remove <id>`**: Delete a task entirely.
- **`edit <id> <field> <value>`**: Edit an existing task (e.g., `todo edit 1 priority critical`).
- **`search <query>`**: Search tasks by title or tag.
- **`stats`**: View a visual breakdown of your task completion.
- **`clear [--done-only]`**: Wipe tasks (add flag to only clear finished ones).
- **`undo`**: Revert the last change made to your list.
- **`redo`**: Re-apply the last undone action.

### Priorities
Supported priority levels:
- `low` or `l`
- `medium`, `med`, or `m`
- `high` or `h`
- `critical` or `c`

## Data Storage

Your tasks are persisted locally in a `todos.json` file in the directory where the command is run. 

## License

MIT
