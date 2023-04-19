# todos

Rust-based CLI todos tracker based on a dependency tree of todos (or rather, a dependency DAG of todos). Only shows you the leaves unless you ask it to overwhelm you.

The idea behind this project is to show you only the tasks on your list that you can complete at the moment. It is to help clear one's mind of the things they must do but are waiting on a condition or the completion of another task.

![super informative alt text](res/example.png "Don't worry, many of these items have been completed since taking this screenshot.")

## Requirements

- `git`
- `cargo` / `rust` toolchain

## Setup:

- *nix / MacOS:
  1. Clone the repo
  2. Whenever you want to update
    - `cd` into repo root
    - pull most recent changes, if any
    - run `update_or_install.sh` (all this script does is run cargo install and handles breaking changes in todos storage file or configuration, if any.)

- Windows
  1. Clone the repo
  2. Read `update_or_uninstall.sh`
  3. Whenever you want to update, try to do the same thing `update_or_uninstall.sh` does manually! :)

Configure console colors (more configurations coming soon) in `~/.todos/config.toml`. Supports basic ANSI color codes (i.e. bold blue "01;94", RGB orange "38;2;255;127;0", or bold RGB orange "1;38;2;255;127;0").

## Types of Nodes

There are three types of nodes:

- `goal`: always shown when displaying todos. Acts as a sort of category or long-term goal. Used for grouping todos. Can be nested.
  - (i.e. "work", "college", "personal projects", "being a responsible person", etc.)
- `condition`: used to indicate something you're waiting on -- something you cannot do, only tell when it's done
  - (i.e. "Jeff has messaged me back", "It's August", etc.)
- `task`: an actionable todo

## Usage:

- show todos:
```bash
# This shows all todos under the current effective root ("root", the hidden root of all nodes by default.)
# See `todos use` for more
todos
```
or, to only display nodes that are an indirect dependency of another
```bash
todos under [id of indirect parent]
```
Add the flag `--overwhelm` (or, equivalently, `-o`) to show all the todos, not just the leaves.
- `add` command:
```bash
# This adds a node to the effective root of the DAG.
todos add [node type (optional, default is "task")] "[description]"
```
or to go ahead and add the node to a parent,
```bash
todos add [node type (optional, default is "task")] "[description]" to [id of direct parent to be]
# or
todos add [node type (optional, default is "task")] "[description]" under [id of direct parent to be]
```

or to insert a node as the direct parent of another,
```bash
todos add [node type (optional, default is "task")] "[description]" above [id of direct child to be]
```
- `complete` command:
```bash
todos complete [id of completed node] [id of completed node] ... [id of completed node]
```
Operates recursively, completing subtasks as well.
- `link` command: for if you want a node you've already added to depend on or be depended on by another node.
```bash
todos link [id of direct parent to be] [id of direct child to be]
```
- `unlink` command: for if you change your mind about the structure of your dependency DAG.
```bash
todos unlink [id of direct parent] [id of direct child]
```

- `use` command: sets the effective root node to the desired node.
```bash
todos use root # the hidden root node, parent of all nodes
# or
todos use [id of new effective root] # a specific node in the DAG
```

- `edit` command: to change the description of a node.
```bash
todos edit [id of node] [new description]
# or
todos relabel [id of node] [new description]
```

- `shell` command: enter shell mode. In shell mode, all commands are identical except `todos` does not need to be prepended to the commands. You can use `<ctrl-d>` to exit the shell (just like any other shell).
```bash
todos shell
```

## Backlog

It may be pleasant to hide the truth of just how much stuff you want to do and never get around to with a `backlog`. By default, any goal named `backlog` will hide its children. If you want to see its contents, you can either `use` it as the effective root or show only todos `under` it.

This functionality can be disabled by setting `hide_backlog_items` to false in the config file. In addition, you can change the name from "backlog" to something else by editing `backlog_name`, also in the config file.


## Config file

The config file is installed in `~/.todos/config.toml`. At this point in time the config file must fit the following schema in TOML format:

```
  hide_backlog_items: bool,
  backlog_name: String,
  goal_color: String,
  condition_color: String,
  task_color: String
```

As things are, I have incorporated automatic updates of the config file in the update script. However, it is the least robust part of the application, and I do not guarantee it is perfect. It may be good to just take a look at the config file after updating to make sure it is how you want it to be.

## Disclaimer

Because it was written originally for myself, this app contains mildly insulting messages. :)

## Upcoming changes
- timed conditions
