//! 🔥 BALLIN' TODO LIST - The Ultimate Task Manager
//! 
//! Features:
//! - 🌈 Color-coded priorities and categories
//! - 📅 Due dates with overdue warnings
//! - 🏷️ Tags for organization
//! - 📊 Progress tracking & statistics
//! - 🔍 Search and filter
//! - ↩️ Undo/Redo support
//! - 🎨 Beautiful terminal UI

use chrono::{DateTime, Local, NaiveDate, Days};
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::Path;

// =============================================================================
// CORE DATA MODELS
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Priority {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "low" | "l" => Some(Priority::Low),
            "medium" | "med" | "m" => Some(Priority::Medium),
            "high" | "h" => Some(Priority::High),
            "critical" | "crit" | "c" | "urgent" | "u" => Some(Priority::Critical),
            _ => None,
        }
    }

    fn to_colored_string(&self) -> ColoredString {
        match self {
            Priority::Low => "● LOW".cyan(),
            Priority::Medium => "● MED".yellow(),
            Priority::High => "● HIGH".red(),
            Priority::Critical => "● CRIT".bright_red().bold(),
        }
    }

    fn value(&self) -> u8 {
        match self {
            Priority::Low => 1,
            Priority::Medium => 2,
            Priority::High => 3,
            Priority::Critical => 4,
        }
    }
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Medium
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: u32,
    pub title: String,
    pub completed: bool,
    pub priority: Priority,
    pub created_at: DateTime<Local>,
    pub completed_at: Option<DateTime<Local>>,
    pub due_date: Option<NaiveDate>,
    pub tags: Vec<String>,
    pub category: Option<String>,
}

impl TodoItem {
    pub fn new(id: u32, title: String) -> Self {
        Self {
            id,
            title,
            completed: false,
            priority: Priority::default(),
            created_at: Local::now(),
            completed_at: None,
            due_date: None,
            tags: Vec::new(),
            category: None,
        }
    }

    pub fn mark_done(&mut self) {
        self.completed = true;
        self.completed_at = Some(Local::now());
    }

    pub fn mark_undone(&mut self) {
        self.completed = false;
        self.completed_at = None;
    }

    pub fn is_overdue(&self) -> bool {
        if self.completed {
            return false;
        }
        if let Some(due) = self.due_date {
            due < Local::now().date_naive()
        } else {
            false
        }
    }

    pub fn due_soon(&self) -> bool {
        if self.completed || self.due_date.is_none() {
            return false;
        }
        let due = self.due_date.unwrap();
        let today = Local::now().date_naive();
        let two_days = today.checked_add_days(Days::new(2)).unwrap();
        due <= two_days && due >= today
    }

    pub fn days_remaining(&self) -> Option<i64> {
        self.due_date.map(|due| {
            (due - Local::now().date_naive()).num_days()
        })
    }
}

// =============================================================================
// CLI PARSING
// =============================================================================

pub mod cli {
    use super::*;

    pub struct Cli {
        pub command: Command,
    }

    pub enum Command {
        Add { title: String, priority: Option<String>, due: Option<String>, tags: Vec<String>, category: Option<String> },
        Remove { id: u32 },
        List { filter: Option<String>, sort: Option<String> },
        MarkDone { id: u32 },
        MarkUndone { id: u32 },
        Edit { id: u32, field: String, value: String },
        Search { query: String },
        Stats,
        Clear { done_only: bool },
        Undo,
        Redo,
        Help,
    }

    pub fn print_usage() {
        println!("{}", "🔥 BALLIN' TODO LIST 🔥".bright_yellow().bold());
        println!();
        println!("{}", "USAGE:".bright_cyan().underline());
        println!("  todo <command> [options]");
        println!();
        println!("{}", "COMMANDS:".bright_cyan().underline());
        println!("  {} <title> [-p priority] [-d date] [-t tags] [-c category]  {}", 
            "add".green().bold(), "Add a new task");
        println!("  {} [filter] [--sort FIELD]                                  {}", 
            "list".green().bold(), "List tasks (filter: done, undone, overdue, due-soon)");
        println!("  {} <id>                                                     {}", 
            "done".green().bold(), "Mark task as complete");
        println!("  {} <id>                                                    {}", 
            "undo-task".green().bold(), "Mark task as incomplete");
        println!("  {} <id>                                                  {}", 
            "remove".green().bold(), "Delete a task");
        println!("  {} <id> <field> <value>                            {}", 
            "edit".green().bold(), "Edit a task field");
        println!("  {} <query>                                             {}", 
            "search".green().bold(), "Search tasks by title/tags");
        println!("  {}                                                       {}", 
            "stats".green().bold(), "Show statistics dashboard");
        println!("  {} [--done-only]                                          {}", 
            "clear".green().bold(), "Clear tasks");
        println!("  {}                                                        {}", 
            "undo".green().bold(), "Undo last action");
        println!("  {}                                                        {}", 
            "redo".green().bold(), "Redo last undone action");
        println!("  {}                                                        {}", 
            "help".green().bold(), "Show this help");
        println!();
        println!("{}", "EXAMPLES:".bright_cyan().underline());
        println!("  todo add \"Buy groceries\" -p high -d 2024-12-31 -t food,shopping");
        println!("  todo list undone --sort priority");
        println!("  todo search \"urgent\"");
        println!();
        println!("{}", "PRIORITIES:".bright_cyan().underline());
        println!("  low (l), medium (m), high (h), critical (c)");
    }

    pub fn parse() -> Cli {
        let mut args = std::env::args().skip(1);

        match args.next() {
            Some(cmd) => match cmd.as_str() {
                "add" | "a" => parse_add(args),
                "remove" | "rm" | "delete" | "del" => parse_remove(args),
                "list" | "ls" | "l" => parse_list(args),
                "done" | "complete" | "mark-done" => parse_mark_done(args),
                "undo-task" | "uncomplete" => parse_mark_undone(args),
                "edit" | "e" => parse_edit(args),
                "search" | "s" | "find" => parse_search(args),
                "stats" | "st" | "stat" => Cli { command: Command::Stats },
                "clear" | "clean" => parse_clear(args),
                "undo" | "u" => Cli { command: Command::Undo },
                "redo" | "r" => Cli { command: Command::Redo },
                "help" | "h" | "--help" | "-h" => Cli { command: Command::Help },
                _ => {
                    eprintln!("{} Unknown command: '{}'", "❌".red(), cmd);
                    std::process::exit(1);
                }
            },
            None => Cli { command: Command::List { filter: None, sort: None } },
        }
    }

    fn parse_add(mut args: impl Iterator<Item = String>) -> Cli {
        // Collect title until we hit a flag (starting with -)
        let mut title_parts = Vec::new();
        while let Some(arg) = args.next() {
            if arg.starts_with('-') {
                // Put it back for flag processing
                // But we can't easily put it back, so let's collect all args first
                let all_args: Vec<String> = std::iter::once(arg).chain(args).collect();
                
                let title = title_parts.join(" ");
                if title.is_empty() {
                    eprintln!("{} Usage: todo add <title> [options]", "❌".red());
                    std::process::exit(1);
                }
                
                return parse_add_flags(title, all_args);
            }
            title_parts.push(arg);
        }
        
        // No flags found
        let title = title_parts.join(" ");
        if title.is_empty() {
            eprintln!("{} Usage: todo add <title> [options]", "❌".red());
            std::process::exit(1);
        }
        
        Cli { command: Command::Add { title, priority: None, due: None, tags: Vec::new(), category: None } }
    }
    
    fn parse_add_flags(title: String, args: Vec<String>) -> Cli {
        let mut priority = None;
        let mut due = None;
        let mut tags = Vec::new();
        let mut category = None;

        let args_vec = args;
        let mut i = 0;
        while i < args_vec.len() {
            match args_vec[i].as_str() {
                "-p" | "--priority" => {
                    if i + 1 < args_vec.len() {
                        priority = Some(args_vec[i + 1].clone());
                        i += 2;
                    } else { i += 1; }
                }
                "-d" | "--due" | "--date" => {
                    if i + 1 < args_vec.len() {
                        due = Some(args_vec[i + 1].clone());
                        i += 2;
                    } else { i += 1; }
                }
                "-t" | "--tags" => {
                    if i + 1 < args_vec.len() {
                        tags = args_vec[i + 1].split(',').map(|s| s.trim().to_string()).collect();
                        i += 2;
                    } else { i += 1; }
                }
                "-c" | "--category" => {
                    if i + 1 < args_vec.len() {
                        category = Some(args_vec[i + 1].clone());
                        i += 2;
                    } else { i += 1; }
                }
                _ => i += 1,
            }
        }

        Cli { command: Command::Add { title, priority, due, tags, category } }
    }

    fn parse_remove(mut args: impl Iterator<Item = String>) -> Cli {
        let id = args.next()
            .and_then(|s| s.parse().ok())
            .expect("Usage: todo remove <id>");
        Cli { command: Command::Remove { id } }
    }

    fn parse_list(mut args: impl Iterator<Item = String>) -> Cli {
        let mut filter = None;
        let mut sort = None;
        
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--sort" | "-s" => {
                    sort = args.next();
                }
                "done" | "completed" => filter = Some("done".to_string()),
                "undone" | "pending" | "todo" => filter = Some("undone".to_string()),
                "overdue" => filter = Some("overdue".to_string()),
                "due-soon" | "soon" => filter = Some("due-soon".to_string()),
                _ => {}
            }
        }
        
        Cli { command: Command::List { filter, sort } }
    }

    fn parse_mark_done(mut args: impl Iterator<Item = String>) -> Cli {
        let id = args.next()
            .and_then(|s| s.parse().ok())
            .expect("Usage: todo done <id>");
        Cli { command: Command::MarkDone { id } }
    }

    fn parse_mark_undone(mut args: impl Iterator<Item = String>) -> Cli {
        let id = args.next()
            .and_then(|s| s.parse().ok())
            .expect("Usage: todo undo-task <id>");
        Cli { command: Command::MarkUndone { id } }
    }

    fn parse_edit(mut args: impl Iterator<Item = String>) -> Cli {
        let id = args.next()
            .and_then(|s| s.parse().ok())
            .expect("Usage: todo edit <id> <field> <value>");
        let field = args.next().expect("Usage: todo edit <id> <field> <value>");
        let value = args.collect::<Vec<_>>().join(" ");
        Cli { command: Command::Edit { id, field, value } }
    }

    fn parse_search(args: impl Iterator<Item = String>) -> Cli {
        let query = args.collect::<Vec<_>>().join(" ");
        if query.is_empty() {
            eprintln!("{} Usage: todo search <query>", "❌".red());
            std::process::exit(1);
        }
        Cli { command: Command::Search { query } }
    }

    fn parse_clear(mut args: impl Iterator<Item = String>) -> Cli {
        let done_only = args.any(|a| a == "--done-only" || a == "-d");
        Cli { command: Command::Clear { done_only } }
    }
}

// =============================================================================
// STORAGE WITH HISTORY (UNDO/REDO)
// =============================================================================

#[derive(Debug)]
pub struct Storage {
    todos: Vec<TodoItem>,
    history: Vec<Vec<TodoItem>>,
    history_pos: usize,
    max_history: usize,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            todos: Vec::new(),
            history: Vec::new(),
            history_pos: 0,
            max_history: 20,
        }
    }

    pub fn load(path: &Path) -> io::Result<Self> {
        let todos = if path.exists() {
            let content = fs::read_to_string(path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        };
        
        Ok(Self {
            todos,
            history: Vec::new(),
            history_pos: 0,
            max_history: 20,
        })
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        let json = serde_json::to_string_pretty(&self.todos)?;
        fs::write(path, json)
    }

    pub fn snapshot(&mut self) {
        // Remove any future history if we're not at the end
        if self.history_pos < self.history.len() {
            self.history.truncate(self.history_pos);
        }
        
        self.history.push(self.todos.clone());
        if self.history.len() > self.max_history {
            self.history.remove(0);
        } else {
            self.history_pos = self.history.len();
        }
    }

    pub fn undo(&mut self) -> bool {
        if self.history_pos == 0 || self.history.is_empty() {
            return false;
        }
        self.history_pos -= 1;
        self.todos = self.history[self.history_pos].clone();
        true
    }

    pub fn redo(&mut self) -> bool {
        if self.history_pos >= self.history.len() {
            return false;
        }
        self.todos = self.history[self.history_pos].clone();
        self.history_pos += 1;
        true
    }

    pub fn todos(&self) -> &Vec<TodoItem> {
        &self.todos
    }

    pub fn todos_mut(&mut self) -> &mut Vec<TodoItem> {
        &mut self.todos
    }
}

// =============================================================================
// COMMANDS
// =============================================================================

pub mod commands {
    use super::*;

    pub fn add_todo(
        storage: &mut Storage,
        title: String,
        priority_str: Option<String>,
        due_str: Option<String>,
        tags: Vec<String>,
        category: Option<String>,
    ) -> Result<String, String> {
        storage.snapshot();
        
        let next_id = storage.todos().iter().map(|t| t.id).max().unwrap_or(0) + 1;
        let mut item = TodoItem::new(next_id, title);
        
        if let Some(p) = priority_str {
            if let Some(priority) = Priority::from_str(&p) {
                item.priority = priority;
            }
        }
        
        if let Some(d) = due_str {
            if let Ok(date) = NaiveDate::parse_from_str(&d, "%Y-%m-%d") {
                item.due_date = Some(date);
            }
        }
        
        item.tags = tags;
        item.category = category;
        
        let msg = format!("{} Added task #{}: {}", "✅".green(), item.id, item.title);
        storage.todos_mut().push(item);
        Ok(msg)
    }

    pub fn remove_todo(storage: &mut Storage, id: u32) -> Result<String, String> {
        storage.snapshot();
        
        if let Some(pos) = storage.todos().iter().position(|t| t.id == id) {
            let item = storage.todos_mut().remove(pos);
            Ok(format!("{} Removed task #{}: {}", "🗑️".red(), id, item.title))
        } else {
            Err(format!("{} Task #{} not found", "❌".red(), id))
        }
    }

    pub fn mark_done(storage: &mut Storage, id: u32) -> Result<String, String> {
        storage.snapshot();
        
        if let Some(item) = storage.todos_mut().iter_mut().find(|t| t.id == id) {
            item.mark_done();
            Ok(format!("{} Completed task #{}: {}", "✅".green(), id, item.title))
        } else {
            Err(format!("{} Task #{} not found", "❌".red(), id))
        }
    }

    pub fn mark_undone(storage: &mut Storage, id: u32) -> Result<String, String> {
        storage.snapshot();
        
        if let Some(item) = storage.todos_mut().iter_mut().find(|t| t.id == id) {
            item.mark_undone();
            Ok(format!("{} Reopened task #{}: {}", "🔄".yellow(), id, item.title))
        } else {
            Err(format!("{} Task #{} not found", "❌".red(), id))
        }
    }

    pub fn edit_todo(storage: &mut Storage, id: u32, field: &str, value: &str) -> Result<String, String> {
        storage.snapshot();
        
        if let Some(item) = storage.todos_mut().iter_mut().find(|t| t.id == id) {
            match field {
                "title" => item.title = value.to_string(),
                "priority" => {
                    if let Some(p) = Priority::from_str(value) {
                        item.priority = p;
                    } else {
                        return Err(format!("{} Invalid priority: {}", "❌".red(), value));
                    }
                }
                "due" | "date" => {
                    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
                        item.due_date = Some(date);
                    } else {
                        return Err(format!("{} Invalid date format. Use YYYY-MM-DD", "❌".red()));
                    }
                }
                "category" => item.category = Some(value.to_string()),
                "tags" => item.tags = value.split(',').map(|s| s.trim().to_string()).collect(),
                _ => return Err(format!("{} Unknown field: {}", "❌".red(), field)),
            }
            Ok(format!("{} Updated task #{} {} → {}", "✏️".cyan(), id, field, value))
        } else {
            Err(format!("{} Task #{} not found", "❌".red(), id))
        }
    }

    pub fn search_todos<'a>(storage: &'a Storage, query: &str) -> Vec<&'a TodoItem> {
        let query_lower = query.to_lowercase();
        storage.todos()
            .iter()
            .filter(|t| {
                t.title.to_lowercase().contains(&query_lower) ||
                t.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower)) ||
                t.category.as_ref().map(|c| c.to_lowercase().contains(&query_lower)).unwrap_or(false)
            })
            .collect()
    }

    pub fn clear_todos(storage: &mut Storage, done_only: bool) -> String {
        storage.snapshot();
        
        if done_only {
            let before = storage.todos().len();
            storage.todos_mut().retain(|t| !t.completed);
            let removed = before - storage.todos().len();
            format!("{} Cleared {} completed tasks", "🧹".cyan(), removed)
        } else {
            let count = storage.todos().len();
            storage.todos_mut().clear();
            format!("{} Cleared all {} tasks", "🧹".cyan(), count)
        }
    }

    pub fn undo(storage: &mut Storage) -> String {
        if storage.undo() {
            format!("{} Undid last action", "↩️".yellow())
        } else {
            format!("{} Nothing to undo", "⚠️".yellow())
        }
    }

    pub fn redo(storage: &mut Storage) -> String {
        if storage.redo() {
            format!("{} Redid last action", "↪️".green())
        } else {
            format!("{} Nothing to redo", "⚠️".yellow())
        }
    }
}

// =============================================================================
// UI / DISPLAY
// =============================================================================

pub fn print_banner() {
    println!();
    println!("{}", "╔═══════════════════════════════════════╗".bright_yellow());
    println!("{}", "║     🔥 BALLIN' TODO LIST v1.0 🔥      ║".bright_yellow().bold());
    println!("{}", "╚═══════════════════════════════════════╝".bright_yellow());
    println!();
}

pub fn print_todos(todos: &[&TodoItem], sort: Option<&str>) {
    if todos.is_empty() {
        println!("{} No tasks found!", "📭".white());
        return;
    }

    let mut sorted: Vec<&&TodoItem> = todos.iter().collect();
    
    match sort {
        Some("priority") => sorted.sort_by(|a, b| b.priority.value().cmp(&a.priority.value())),
        Some("date") => sorted.sort_by(|a, b| a.due_date.cmp(&b.due_date)),
        Some("created") => sorted.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
        _ => {}
    }

    // Header
    println!("{}", "┌─────┬─────────────────────────────────────────────────────────────┬──────────┬──────────────┬────────────┐".bright_black());
    println!("{}", "│ ID  │ TITLE                                                       │ PRIORITY │ DUE DATE     │ STATUS     │".bright_black());
    println!("{}", "├─────┼─────────────────────────────────────────────────────────────┼──────────┼──────────────┼────────────┤".bright_black());

    for item in sorted {
        let status = if item.completed {
            "✅ DONE".green()
        } else if item.is_overdue() {
            "⚠️ OVERDUE".red().bold()
        } else if item.due_soon() {
            "⏰ SOON".yellow()
        } else {
            "⏳ PENDING".white()
        };

        let due_str = if let Some(days) = item.days_remaining() {
            if item.completed {
                "-".to_string()
            } else if days < 0 {
                format!("{} ({}d ago)", item.due_date.unwrap(), days.abs()).red().to_string()
            } else if days == 0 {
                format!("{} TODAY", item.due_date.unwrap()).yellow().to_string()
            } else if days == 1 {
                format!("{} (tomorrow)", item.due_date.unwrap()).yellow().to_string()
            } else {
                format!("{} ({}d)", item.due_date.unwrap(), days).to_string()
            }
        } else {
            "-".to_string()
        };

        let title_display = if item.title.len() > 55 {
            format!("{}...", &item.title[..52])
        } else {
            item.title.clone()
        };

        println!(
            "{} {:>3} {} {:<59} {} {:>8} {} {:>12} {} {:>10} {}",
            "│".bright_black(),
            item.id,
            "│".bright_black(),
            title_display,
            "│".bright_black(),
            item.priority.to_colored_string(),
            "│".bright_black(),
            due_str,
            "│".bright_black(),
            status,
            "│".bright_black()
        );

        // Tags line
        if !item.tags.is_empty() || item.category.is_some() {
            let tag_str = item.tags.iter()
                .map(|t| format!("#{}", t))
                .chain(item.category.iter().cloned())
                .collect::<Vec<_>>()
                .join(" ");
            
            let tag_display = if tag_str.len() > 55 {
                format!("{}...", &tag_str[..52])
            } else {
                tag_str
            };
            
            println!(
                "{}     {} {:<59} {}          {}              {}            {}",
                "│".bright_black(),
                "│".bright_black(),
                tag_display.bright_cyan(),
                "│".bright_black(),
                "│".bright_black(),
                "│".bright_black(),
                "│".bright_black()
            );
        }
    }

    println!("{}", "└─────┴─────────────────────────────────────────────────────────────┴──────────┴──────────────┴────────────┘".bright_black());
}

pub fn print_stats(storage: &Storage) {
    let todos = storage.todos();
    let total = todos.len();
    let done = todos.iter().filter(|t| t.completed).count();
    let pending = total - done;
    let overdue = todos.iter().filter(|t| t.is_overdue()).count();
    let due_soon = todos.iter().filter(|t| t.due_soon()).count();
    
    let low = todos.iter().filter(|t| !t.completed && t.priority == Priority::Low).count();
    let med = todos.iter().filter(|t| !t.completed && t.priority == Priority::Medium).count();
    let high = todos.iter().filter(|t| !t.completed && t.priority == Priority::High).count();
    let crit = todos.iter().filter(|t| !t.completed && t.priority == Priority::Critical).count();

    println!();
    println!("{}", "╔══════════════════════════════════════════════════════════╗".bright_cyan());
    println!("{}", "║              📊 TASK STATISTICS DASHBOARD                ║".bright_cyan().bold());
    println!("{}", "╚══════════════════════════════════════════════════════════╝".bright_cyan());
    println!();

    // Progress bar
    let progress = if total > 0 { done as f64 / total as f64 } else { 0.0 };
    let bar_width = 40;
    let filled = (progress * bar_width as f64) as usize;
    let empty = bar_width - filled;
    
    let bar = format!(
        "{}{}",
        "█".repeat(filled).bright_green(),
        "░".repeat(empty).bright_black()
    );
    
    println!("  {}  {}%", bar, (progress * 100.0) as u32);
    println!("  {}/{} tasks completed", done, total);
    println!();

    // Stats grid
    println!("  {} {}", "📋 Total:".bright_white(), total);
    println!("  {} {}", "✅ Done:".green(), done);
    println!("  {} {}", "⏳ Pending:".yellow(), pending);
    println!("  {} {}", "⚠️  Overdue:".red(), overdue);
    println!("  {} {}", "⏰ Due Soon:".yellow(), due_soon);
    println!();

    // Priority breakdown
    println!("  {}", "PRIORITY BREAKDOWN:".bright_cyan().underline());
    println!("    {} Low     │ {}", "●".cyan(), low);
    println!("    {} Medium  │ {}", "●".yellow(), med);
    println!("    {} High    │ {}", "●".red(), high);
    println!("    {} Critical│ {}", "●".bright_red(), crit);
    println!();

    // Categories
    let mut categories: HashSet<String> = HashSet::new();
    for t in todos.iter().filter(|t| !t.completed) {
        if let Some(cat) = &t.category {
            categories.insert(cat.clone());
        }
    }
    
    if !categories.is_empty() {
        println!("  {}", "CATEGORIES:".bright_cyan().underline());
        for cat in categories {
            let count = todos.iter().filter(|t| t.category.as_ref() == Some(&cat) && !t.completed).count();
            println!("    📁 {}: {}", cat.bright_magenta(), count);
        }
        println!();
    }
}

// =============================================================================
// MAIN
// =============================================================================

fn main() {
    let path = Path::new("todos.json");
    let mut storage = Storage::load(path).unwrap_or_else(|_| Storage::new());
    
    let cli = cli::parse();
    
    let result = match cli.command {
        cli::Command::Add { title, priority, due, tags, category } => {
            commands::add_todo(&mut storage, title, priority, due, tags, category)
        }
        cli::Command::Remove { id } => {
            commands::remove_todo(&mut storage, id)
        }
        cli::Command::List { filter, sort } => {
            let filtered: Vec<&TodoItem> = match filter.as_deref() {
                Some("done") => storage.todos().iter().filter(|t| t.completed).collect(),
                Some("undone") => storage.todos().iter().filter(|t| !t.completed).collect(),
                Some("overdue") => storage.todos().iter().filter(|t| t.is_overdue()).collect(),
                Some("due-soon") => storage.todos().iter().filter(|t| t.due_soon()).collect(),
                _ => storage.todos().iter().collect(),
            };
            
            print_banner();
            print_todos(&filtered, sort.as_deref());
            Ok("".to_string())
        }
        cli::Command::MarkDone { id } => {
            commands::mark_done(&mut storage, id)
        }
        cli::Command::MarkUndone { id } => {
            commands::mark_undone(&mut storage, id)
        }
        cli::Command::Edit { id, field, value } => {
            commands::edit_todo(&mut storage, id, &field, &value)
        }
        cli::Command::Search { query } => {
            let results = commands::search_todos(&storage, &query);
            print_banner();
            println!("{} Searching for: '{}'", "🔍".cyan(), query.cyan());
            println!();
            print_todos(&results, None);
            Ok("".to_string())
        }
        cli::Command::Stats => {
            print_stats(&storage);
            Ok("".to_string())
        }
        cli::Command::Clear { done_only } => {
            Ok(commands::clear_todos(&mut storage, done_only))
        }
        cli::Command::Undo => {
            Ok(commands::undo(&mut storage))
        }
        cli::Command::Redo => {
            Ok(commands::redo(&mut storage))
        }
        cli::Command::Help => {
            cli::print_usage();
            Ok("".to_string())
        }
    };

    match result {
        Ok(msg) if !msg.is_empty() => println!("{}", msg),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
        _ => {}
    }

    if let Err(e) = storage.save(path) {
        eprintln!("{} Failed to save: {}", "💾".red(), e);
        std::process::exit(1);
    }
}
