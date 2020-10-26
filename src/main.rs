// #[macro_use] extern crate num_derive;

mod node;
use node::{NodeType, Node, Graph};
use std::env;

pub enum Command
{
    New
    {
        description: String,
        node_type: NodeType,
        to: Option<usize>
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
    }
}

fn main()
{
    let args: Vec<String> = env::args().collect();
    let mut g = Graph::load("/home/nelson/.todos", node::StorageVersion::JSON);
    match get_command(args) 
    {
        Some(command) => { parse_command(command, &mut g) },
        None => { println!("Invalid command."); }
    }
    g.save();
}
fn get_command(arg_list: Vec<String>) -> Option<Command>
{
    let mut overwhelm = false;
    let mut sanitized = Vec::<String>::new();
    for i in 0..arg_list.len()
    {
        let arg = &arg_list[i];
        if arg == "-o" || arg == "--overwhelm"
        {
            overwhelm = true;
        }
        else if i != 0
        {
            sanitized.push(arg.clone());
        }
    };
    if sanitized.is_empty()
    {
        return Some(Command::Show{ overwhelm: overwhelm });
    }
    match &sanitized[0][..]
    {
        "add" => {
            match sanitized.len()
            {
                2 => {
                    let node_type = NodeType::TASK;
                    let description = sanitized[1].clone();
                    Some(Command::New{description:description, node_type:node_type, to:None})
                },
                3 => {
                    let node_type = match NodeType::from_string(&sanitized[1])
                    {
                        Some(nt) => nt,
                        None => { return None; }
                    };
                    let description = sanitized[2].clone();
                    Some(Command::New{description:description, node_type:node_type, to:None})
                },
                4 => {
                    let node_type = NodeType::TASK;
                    let description = sanitized[1].clone();
                    let to: usize = match &sanitized[2][..]
                    {
                        "to" => {
                            match sanitized[3].parse::<usize>()
                            {
                                Ok(val) => val,
                                Err(_error) => { return None; }
                            }
                        },
                        _ => { return None; } 
                    };
                    Some(Command::New{description:description, node_type:node_type, to:Some(to)})
                },
                5 => {
                    let node_type = match NodeType::from_string(&sanitized[1])
                    {
                        Some(nt) => nt,
                        None => { return None; }
                    };
                    let description = sanitized[2].clone();
                    let to: usize = match &sanitized[3][..]
                    {
                        "to" => {
                            match sanitized[4].parse::<usize>()
                            {
                                Ok(val) => val,
                                Err(_error) => { return None; }
                            }
                        },
                        _ => { return None; } 
                    };
                    Some(Command::New{description:description, node_type:node_type, to:Some(to)})

                },
                _ => { return None; }
            }
        },
        "complete" => 
        {
            match sanitized.len()
            {
                2 => {
                    let id: usize = match sanitized[1].trim().parse::<usize>()
                    {
                        Ok(val) => val,
                        Err(_error)=> {
                            return None;
                        }
                    };
                    Some(Command::Complete{ id })
                },
                _ => { return None; }
            }
        },
        "link" => 
        {
            match sanitized.len()
            {
                3 => {
                    let parent: usize = match sanitized[1].trim().parse::<usize>()
                    {
                        Ok(val) => val,
                        Err(_error)=> {
                            return None;
                        }
                    };
                    let child: usize = match sanitized[2].trim().parse::<usize>()
                    {
                        Ok(val) => val,
                        Err(_error)=> {
                            return None;
                        }
                    };
                    Some(Command::Link{ parent, child })
                },
                _ => { return None; }
            }
        },
        "unlink" => 
        {
            match sanitized.len()
            {
                3 => {
                    let parent: usize = match sanitized[1].trim().parse::<usize>()
                    {
                        Ok(val) => val,
                        Err(_error)=> {
                            return None;
                        }
                    };
                    let child: usize = match sanitized[2].trim().parse::<usize>()
                    {
                        Ok(val) => val,
                        Err(_error)=> {
                            return None;
                        }
                    };
                    Some(Command::Unlink{ parent, child })
                },
                _ => { return None; }
            }
        },
        "under" =>
        {
            match sanitized.len()
            {
                2 => {
                    let id: usize = match sanitized[1].trim().parse::<usize>()
                    {
                        Ok(val) => val,
                        Err(_error)=> {
                            return None;
                        }
                    };
                    Some(Command::Under{ id, overwhelm })
                },
                _ => { return None; }
            }

        },
        _ => { return None; }
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
                    graph.nodes[id].print(1);

                },
                Err(()) => {
                    println!("Idiot. Haven't you heard of a DAG before?");
                }
            }
        },
        Command::Complete { id } => {
            match graph.remove_node(id, true)
            {
                Ok(()) => {
                    println!("Thank god, you managed to complete something")
                },
                Err(()) => {
                    println!("Failed to remove node.");
                }
            }
        },
        Command::Link { parent, child } => {
            match graph.link(&parent, &child)
            {
                Ok(()) => {
                    println!("Successfully created link");
                }
                Err(()) => {
                    println!("Idiot. Haven't you heard of a DAG before?");
                }
            }
        },
        Command::Unlink { parent, child } => {
            match graph.unlink(&parent, &child)
            {
                Ok(()) => {
                    println!("Successfully removed link");
                }
                Err(()) => {
                    println!("Idiot. Haven't you heard of a DAG before?");
                }
            }
        },
        Command::Show { overwhelm } => {
            graph.todos(overwhelm);
        },
        Command::Under { id, overwhelm } => {
            graph.show(&id, 0, overwhelm)
        }
    }
}
