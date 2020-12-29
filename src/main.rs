extern crate dirs;

mod node;
mod graph;
mod config;

use node::{NodeType, Node};
use graph::Graph;
use std::env;
use std::collections::VecDeque;
use std::path::Path;


pub enum Command
{
    New
    {
        description: String,
        node_type: NodeType,
        to: Option<usize>,
    },
    NewBefore
    {
        description: String,
        node_type: NodeType,
        before: usize,
    },
    Complete
    {
        id: usize
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
    Edit
    {
        id: usize,
        new_description: String
    }
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

    match get_command(&mut args) 
    {
        Some(command) => { parse_command(command, &mut graph) },
        None => { println!("Invalid command."); }
    }
    graph.save();
}



fn get_command(arg_list: &mut VecDeque<String>) -> Option<Command>
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
                    return Some(Command::NewBefore{
                        node_type: node_type,
                        description: description,
                        before: id
                    });
                },
                _ => {
                    return None;
                }
            }

        },
        "complete" => {
            let id = arg_list.pop_front()?.parse::<usize>().ok()?;
            if !arg_list.is_empty()
            {
                return None;
            }
            return Some(Command::Complete{
                id: id
            });

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
        _ => {
            return None
        }
    }

}

fn parse_command(command: Command, graph: &mut Graph)
{
    match command
    {
        Command::New { description, node_type, to } => {
            let n = Node
            {
                id:0,
                description: description,
                node_type: node_type,
                due_date: None,
                deps: Vec::<usize>::new(),
                parents: match to {
                    Some(val) => vec![val],
                    None => Vec::<usize>::new()
                }
            };
            match graph.add_node(n)
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
        Command::NewBefore { description, node_type, before } => {
            let node_to_shift = &graph.nodes[before];
            let parents = node_to_shift.parents.clone();
            // unlink all references before node 'before'
            for parent in &parents
            {
                match graph.unlink(&parent, &before){
                    Ok(()) => {},
                    Err(message) => {
                        panic!(message);
                    }
                }
                
            }

            let n = Node
            {
                id:0,
                description: description,
                node_type: node_type,
                due_date: None,
                deps: vec![before],
                parents: parents
            };

            match graph.add_node(n)
            {
                Ok(id) => {
                    match graph.link(&id, &before)
                    {
                        Ok(()) => {
                            println!("Ha! Your workload just got a little bigger. Node added:"); 
                            graph.print_node(id, 1).unwrap();
                        },
                        Err(message) => {
                            println!("{}", message);
                        }

                    }

                },
                Err(message) => {
                    println!("{}", message);
                }
            }

        },
        Command::Complete { id } => {
            match graph.remove_node(id, true)
            {
                Ok(()) => {
                    println!("Thank god, you managed to complete something")
                },
                Err(message) => {
                    println!("{}", message);
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
            graph.show(&id, 0, overwhelm)
        },
        Command::Edit { id, new_description } => 
        {
            match graph.relabel(id, new_description)
            {
                Ok(()) => println!("Successfully relabeled node."),
                Err(message) => println!("{}", message)
            }
        }
    }
}
