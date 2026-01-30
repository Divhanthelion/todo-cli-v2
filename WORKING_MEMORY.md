# WORKING MEMORY

Cross-module knowledge base. Each module leaves notes for modules that depend on it.

## How to Read This File
When implementing a module, find the sections for your dependencies and pay attention to:
- Method signatures (especially return types: Option vs Result, &T vs T)
- Trait implementations you can rely on (FromStr, Clone, etc.)
- Gotchas and non-obvious patterns

## How Notes Are Structured
Each module section contains:
- **Key Types**: The main structs/enums and their purpose
- **Critical Signatures**: Method signatures that are easy to get wrong
- **Trait Impls**: What traits are implemented (use these!)
- **Gotchas**: Things that will break your code if you assume wrong

---

## core

**Trait implementations:**
- `none`

**Key signatures:**
- ``pub fn new(id: u32`
- `title: String) -> Self``
- ``pub fn mark_done(&mut self)``

## storage

**Trait implementations:**
- `none`

**Key signatures:**
- ``pub fn load_todos(path: &Path) -> Result<Vec<TodoItem>`
- `AppError>``
- ``pub fn save_todos(path: &Path`
- `todos: &[TodoItem]) -> Result<()`
- `AppError>``

## cli

**Trait implementations:**
- `none`

**Key signatures:**
- ``pub fn parse() -> Cli`   `pub struct Cli { pub command: Command }`   `pub enum Command { Add { title: String }`
- `Remove { id: u32 }`
- `List`
- `MarkDone { id: u32 } }``

