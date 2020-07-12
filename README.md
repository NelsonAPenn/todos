# todos

Rust-based CLI todos tracker based on a dependency tree of todos (or rather, a dependency DAG of todos). Only shows you the leaves unless you ask it to overwhelm you.

The idea behind this project is to show you only the tasks on your list that you can complete at the moment. It is to help clear one's mind of the things he/she must do but are waiting on a condition or the completion of another task.

## Setup:
Figure it out yourself until I try (maybe someday) to write directions. *Hint, change the paths in node.rs to your home folder and add the release-built executable to `/usr/local/bin`*

## Types of Nodes

There are three types of nodes:

- `goal`: always shown when displaying todos. Acts as a sort of category or long-term goal. Used for grouping todos. Can be nested.
- `condition`: used to indicate something you're waiting on
- `task`: a todo

## Usage:

- show todos:
```bash
todos
```
or, to only display nodes that are an indirect dependency of another
```bash
todos under [id of indirect parent]
```
Add the flag `--overwhelm` (or, equivalently, `-o`) to show all the todos, not just the leaves.
- `add` command:
```bash
todos add [node type (optional, default is a task)] "[description]"
```
or to go ahead and add the node to a parent,
```bash
todos add [node type (optional, default is a task)] "[description]" to [id of direct parent to be]
```
- `complete` command:
```bash
todos complete [id of completed node]
```
Operates recursively, completing subtasks as well.
- `link` command:
```bash
todos link [id of direct parent to be] [id of direct child to be]
```
- `unlink` command:
```bash
todos unlink [id of direct parent] [id of direct child]
```

## Upcoming changes
- deadlines
- Zenity notifications for Ubuntu