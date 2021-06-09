extern crate dirs;

mod node;
mod graph;
mod config;

use node::{NodeType};
use graph::Graph;
use std::env;
use std::collections::VecDeque;
use std::path::Path;
use std::io::{BufReader, BufRead, stdin};


pub enum Command
{
    New
    {
        description: String,
        node_type: NodeType,
        to: Option<usize>,
    },
    NewAbove
    {
        description: String,
        node_type: NodeType,
        above: usize,
    },
    Complete
    {
        node_ids: Vec<usize>
    },
    Link
    {
        parent: usize,
        child: usize
    },
    Unlink
    {
        parent: usize,
        child: usize
    },
    Show
    {
        overwhelm: bool
    },
    Under
    {
        id: usize,
        overwhelm: bool
    },
    Use
    {
        effective_root: Option<usize>
    },
    Edit
    {
        id: usize,
        new_description: String
    },
    Shell
}

fn main()
{
    let mut args: VecDeque<String> = env::args().collect();
    args.pop_front();

    let home_dir = dirs::home_dir().unwrap();
    let root_path = Path::new(&home_dir).join(".todos");
    let todos_file = root_path.join("todos");
    let config_path = root_path.join("config.toml");

    let config = config::read_config_file(config_path);
    let mut graph = Graph::load(todos_file, config);

    match get_command(args) 
    {
        Some(command) => { perform_command(command, &mut graph, false) },
        None => { println!("Invalid command."); }
    }

    graph.save();
}

fn shell_mode(graph: &mut Graph)
{
    let mut line = String::new();
    let stdin = stdin();
    let stdin = stdin.lock();
    let mut reader = BufReader::new(stdin);

    loop
    {
        // get line of input
        let bytes_read = reader.read_line(&mut line).unwrap();
        if bytes_read == 0
        {
            break;
        }

        // split line and parse to get_command
        let args = line
            .split_ascii_whitespace()
            .map(|x| String::from(x))
            .collect::<VecDeque<String>>();

        match get_command(args) 
        {
            Some(command) => { perform_command(command, graph, true) },
            None => { println!("Invalid command."); }
        }

        // could technically be moved outside of the loop; however,
        // in the case of a crash, it is desirable to have the graph
        // saved already.
        graph.save();

        line.clear();
    }

}



fn get_command(mut arg_list: VecDeque<String>) -> Option<Command>
{
    let short = String::from("-o");
    let long = String::from("--overwhelm");
    let overwhelm = arg_list.contains(&short) || arg_list.contains(&long);
    arg_list.retain( |x| *x != short && *x != long);

    let command_option = arg_list.pop_front();
    if let None = command_option
    {
        return Some(Command::Show{
            overwhelm: overwhelm
        });
    };
    let command = command_option.unwrap();
    match &command[..]
    {
        "add" => {
            // read type (default = task)
            let node_type = 
            if let Some(provided) = NodeType::from_string(arg_list.front()?)
            {
                arg_list.pop_front();
                provided
            }
            else
            {
                NodeType::Task
            };
            // read message
            let description = arg_list.pop_front()?;

            // read optional "to" clause or "above" clause
            let next_token_option = arg_list.pop_front();
            if let None = next_token_option
            {
                    return Some(Command::New{
                        node_type: node_type,
                        description: description,
                        to: None
                    });
            }
            let next_token = next_token_option.unwrap();
            match &next_token[..]
            {
                "to" | "under" => {
                    let id = arg_list.pop_front()?.parse::<usize>().ok()?;
                    if !arg_list.is_empty()
                    {
                        return None;
                    }
                    return Some(Command::New{
                        node_type: node_type,
                        description: description,
                        to: Some(id)
                    });


                },
                "above" => {
                    let id = arg_list.pop_front()?.parse::<usize>().ok()?;
                    if !arg_list.is_empty()
                    {
                        return None;
                    }
                    return Some(Command::NewAbove{
                        node_type: node_type,
                        description: description,
                        above: id
                    });
                },
                _ => {
                    return None;
                }
            }

        },
        "complete" => {
            let mut node_ids = vec![];

            while let Some(token) = arg_list.pop_front()
            {
                node_ids.push(token.parse().ok()?);
            }

            if node_ids.is_empty()
            {
                None
            }
            else
            {
                Some(Command::Complete{node_ids})
            }
        },
        "link" => {
            let parent = arg_list.pop_front()?.parse::<usize>().ok()?;
            let child = arg_list.pop_front()?.parse::<usize>().ok()?;
            if !arg_list.is_empty()
            {
                return None;
            }
            return Some( Command::Link{
                parent: parent,
                child: child
            });

        },
        "unlink" => {
            let parent = arg_list.pop_front()?.parse::<usize>().ok()?;
            let child = arg_list.pop_front()?.parse::<usize>().ok()?;
            if !arg_list.is_empty()
            {
                return None;
            }
            return Some( Command::Unlink{
                parent: parent,
                child: child
            });
        },
        "under" =>
        {
            let id = arg_list.pop_front()?.parse::<usize>().ok()?;
            if !arg_list.is_empty()
            {
                return None;
            }
            return Some(Command::Under{
                id: id,
                overwhelm: overwhelm
            });
        }
        "use" =>
        {
            let token = arg_list.pop_front()?;
            let effective_root = match &token[..]
            {
                "root" => { None },
                _ => {
                    Some(token.parse().ok()?)
                }
            };

            if !arg_list.is_empty()
            {
                return None;
            }
            return Some(Command::Use{ effective_root });
        }
        "edit" | "relabel" =>
        {
            let id = arg_list.pop_front()?.parse::<usize>().ok()?;
            let description = arg_list.pop_front()?;
            if !arg_list.is_empty()
            {
                return None;
            }
            return Some(Command::Edit{
                id: id,
                new_description: description
            });
        }
        "shell" =>
        {
            if !arg_list.is_empty()
            {
                None
            }
            else
            {
                Some(Command::Shell)
            }

        }
        _ => {
            return None
        }
    }

}

fn perform_command(command: Command, graph: &mut Graph, in_shell: bool)
{
    match command
    {
        Command::New { description, node_type, to } => {
            match graph.add_node_to(description, node_type, to)
            {
                Ok(id) => {
                    println!("Ha! Your workload just got a little bigger. Node added:"); 
                    graph.print_node(id, 1).unwrap();

                },
                Err(message) => {
                    println!("{}", message);
                }
            }
        },
        Command::NewAbove { description, node_type, above } => {

            match graph.add_node_above(description, node_type, above)
            {
                Ok(id) => {
                    println!("Ha! Your workload just got a little bigger. Node added:"); 
                    graph.print_node(id, 1).unwrap();
                },
                Err(message) => {
                    println!("{}", message);
                }
            }

        },
        Command::Complete { node_ids } => {
            match graph.batch_remove(node_ids, true)
            {
                Ok(()) => {
                    println!("Thank god, you managed to complete something")
                },
                Err(message) => {
                    print!("{}", message);
                }
            }
        },
        Command::Link { parent, child } => {
            match graph.link(&parent, &child)
            {
                Ok(()) => {
                    println!("Successfully created link");
                }
                Err(message) => {
                    println!("{}", message);
                }
            }
        },
        Command::Unlink { parent, child } => {
            match graph.unlink(&parent, &child)
            {
                Ok(()) => {
                    println!("Successfully removed link");
                }
                Err(message) => {
                    println!("{}", message);
                }
            }
        },
        Command::Show { overwhelm } => {
            graph.todos(overwhelm);
        },
        Command::Under { id, overwhelm } => {
            match graph.show(&id, 0, overwhelm, Some(id))
            {
                Err(message) => {
                    println!("{}", message);
                },
                _ => {}
            }
        },
        Command::Use { effective_root } => {
            match graph.set_effective_root(effective_root)
            {
                Err(message) => {
                    println!("{}", message);
                },
                _ => {
                    if let Some(effective_root) = effective_root
                    {
                        println!("Now using {}.", effective_root);
                    }
                    else
                    {
                        println!("Now using root.");
                    }
                }
            }
        },
        Command::Edit { id, new_description } => 
        {
            match graph.relabel(id, new_description)
            {
                Ok(()) => println!("Successfully relabeled node."),
                Err(message) => println!("{}", message)
            }
        }
        Command::Shell => 
        {
            // the following could be recursive if already in shell mode.
            // However, this may not be an issue, as many shells are implemented as such.
            if !in_shell
            {
                shell_mode(graph);
            }
        }
    }
}
