extern crate num;
extern crate serde_json;
extern crate serde;

use std::fmt;
use std::fs;
use std::result::Result as Result;
use std::io::prelude::*;
use std::fs::File;
use num_traits::{FromPrimitive};
use num_derive::{FromPrimitive, ToPrimitive};    
use std::io::BufReader;
use serde::{Serialize, Deserialize};

pub enum StorageVersion
{
    Json,
    HalfCsv
}

#[derive(Clone, Serialize, Deserialize, ToPrimitive, FromPrimitive)]
pub enum NodeType
{
    Task,
    Condition,
    Goal
}
impl fmt::Display for NodeType
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}", match self {
            NodeType::Task => "task",
            NodeType::Condition => "condition",
            NodeType::Goal => "goal"
        })
    }
}
impl NodeType
{
    pub fn from_string(s: &String) -> Option<NodeType>
    {
        match &s[..]
        {
            "task" => Some(NodeType::Task),
            "condition" => Some(NodeType::Condition),
            "goal" => Some(NodeType::Goal),
            _ => None
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Node
{
    pub id: usize,
    pub description: String,
    pub node_type: NodeType,
    pub due_date: Option<String>,
    pub deps: Vec<usize>,
    pub parents: Vec<usize>
}

impl fmt::Display for Node
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "{},{},{},{},{}", self.id, self.description, num::ToPrimitive::to_usize(&self.node_type).unwrap(), self.deps.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(" "), self.parents.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(" "))
    }

}

impl Node
{
    pub fn print(&self, level: u128)
    {
        for _i in 0..level
        {
            print!("  ");
        }
        match self.node_type {
            NodeType::Goal => {
                println!("\x1B[01;94m{} goal (id {}):\x1B[00m", &self.description, &self.id);
            },
            NodeType::Condition => {
                println!("\x1B[01;33m{} ({}): {}\x1B[00m", &self.id, self.node_type ,&self.description);
            },
            _ => {
                println!("{} ({}): {}", &self.id, self.node_type ,&self.description);
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Graph
{
    pub nodes: Vec<Node>,

    #[serde(skip_serializing, skip_deserializing)]
    location: String 
}

#[allow(dead_code)]
impl Graph
{
    pub fn load(filename: &str, version: StorageVersion) ->  Graph
    {
        let mut graph = match version
        {
            StorageVersion::Json => serde_json::from_str(fs::read_to_string(&filename).unwrap().as_str()).unwrap(),
            StorageVersion::HalfCsv => Graph::load_from_half_csv(filename)
        };
        graph.location = String::from(filename);
        graph
    }

    pub fn load_from_half_csv(filename: &str) -> Graph
    {
        let mut nodes:Vec<Node> = Vec::<Node>::new();
        let f = match File::open(filename) {
            Ok(file) => file,
            Err(_error) => {panic!("Unable to open file")}
        };
        let f = BufReader::new(f);
        for line in f.lines()
        {
            let unwrapped = line.unwrap();
            let split: Vec<&str> = unwrapped.split(",").collect();

            assert_eq!(split.len(), 5);
            let id = split[0].trim().parse::<usize>().unwrap();
            let description = String::from(split[1]);
            let node_type:NodeType = FromPrimitive::from_usize(split[2].trim().parse::<usize>().unwrap()).unwrap();
            let deps:Vec<usize> = match split[3].trim(){
                "" => Vec::<usize>::new(),
                _ => split[3].split(" ").map(|x| x.trim().parse::<usize>().unwrap()).collect()
            };
            let parents:Vec<usize> = match split[4].trim(){
                "" => Vec::<usize>::new(),
                _ => split[4].split(" ").map(|x| x.trim().parse::<usize>().unwrap()).collect()
            };
            nodes.push(Node{
                id: id,
                description: description,
                node_type: node_type,
                due_date: None,
                deps: deps,
                parents: parents
            });

        }
        let graph = Graph {
            nodes: nodes,
            location: String::from("")
        };
        match graph.validate()
        {
            Ok(()) => {},
            Err(message) => { panic!(message) }
        };
        graph
    }
    pub fn validate(&self) -> Result<(), String>
    {
        // check indices/ ids are all within bounds
        for i in 0..self.nodes.len()
        {
            // id should be the index of the node
            if self.nodes[i].id != i
            {
                return Err("Ids do not match the list structure.".to_string());
            }
            // indices should all be within bounds
            match self.validate_node(&self.nodes[i])
            {
                Err(message) => { return Err(message); },
                _ => {}
            }
            //links should be double sided
            for p in &self.nodes[i].parents
            {
                if !self.nodes[*p].deps.contains(&i)
                {
                    return Err(format!("One-sided dependency encountered: parent with id {} does not acknowledge the claimed child status of node with id {}", p, i).to_string());
                }
            }
            for p in &self.nodes[i].deps
            {
                if !self.nodes[*p].parents.contains(&i)
                {
                    return Err(format!("One-sided dependency encountered: child with id {} does not acknowledge the claimed parental status of node with id {}", p, i).to_string());
                }
            }
        }
        match self.check_topology()
        {
            Err(message) => { return Err(message) },
            _ => {}
        }
        Ok(())
    }
    fn validate_node(&self, n: &Node) -> Result<(), String>
    {
        for j in &n.deps
        {
            match self.nodes.get(*j)
            {
                None => { return Err(format!("Listed dependency {} for node with id {} not present in graph.", j, n.id ).to_string()); },
                _ => {}
            }
        }
        for j in &n.parents
        {
            match self.nodes.get(*j)
            {
                None => { return Err(format!("Listed parent {} for node with id {} not present in graph.", j, n.id ).to_string()); },
                _ => {}
            }
        }
        return Ok(());
    }
    pub fn add_node(&mut self, n: Node) -> Result<usize, String>
    {
        let id_to_return: usize = self.nodes.len();
        // if it don't work, don't panic
        match self.validate_node(&n)
        {
            Err(message) => {return Err(message);},
            Ok(()) => {
                let node_to_add = Node {
                    id: self.nodes.len(),
                    description: n.description,
                    node_type: n.node_type,
                    due_date: None,
                    deps: n.deps,
                    parents: n.parents
                };

                // add other ends of links
                for p in &node_to_add.parents
                {
                    if !self.nodes[*p].deps.contains(p)
                    {
                        self.nodes[*p].deps.push(id_to_return.clone());
                    }
                }
                for p in &node_to_add.deps
                {
                    if !self.nodes[*p].parents.contains(p)
                    {
                        self.nodes[*p].parents.push(id_to_return.clone());
                    }
                }
                // insert node
                self.nodes.push(node_to_add);
            }
        }
        match self.validate()
        {
            Ok(()) => { return Ok(id_to_return); },
            Err(_message) => {
                match self.remove_node(self.nodes.len() - 1, false) {
                    Ok(()) => {},
                    Err(message) => panic!(message)
                }
            } // try to remove last node
        }
        // but if it messes up your graph, then panic
        match self.validate()
        {
            Ok(()) => { return Ok(id_to_return); },
            Err(message) => { panic!("Added crappy node and couldn't properly get rid of it!!!\n Error: {}", message); } 
        }
    }
    pub fn remove_node(&mut self, index: usize, recurse: bool) -> Result<(), String>
    {
        match self.nodes.get(index)
        {
            None => { return Err(format!("Node with id {} not present in todos.", index).to_string()); },
            _ => {}
        }

        self.inner_remove(index.clone(), recurse);

        let invalid_val = self.nodes.len().clone();
        //remove invalid nodes
        self.nodes.retain(|x| x.id != invalid_val);

        //update ids/indices
        for i in 0..self.nodes.len()
        {
            self.rename_node(&self.nodes[i].id.clone(), &i);
        }
        // but if it messes up your graph, then panic
        match self.validate()
        {
            Ok(()) => { return Ok(()); },
            Err(message) => { panic!(message); } 
        }
    }

    fn rename_node(&mut self, old_id: &usize, new_id: &usize)
    {
        for node in &mut self.nodes
        {
            if node.id == *old_id
            {
                node.id = new_id.clone();
            }

            node.parents = node.parents.iter().map(|x| { if x == old_id { new_id.clone() } else { x.clone() } }).collect();
            node.deps = node.deps.iter().map(|x| { if x == old_id { new_id.clone() } else { x.clone() } }).collect();
        }
    }

    fn inner_remove(&mut self, index: usize, recurse: bool)
    {

        let to_remove:Vec<usize> = self.nodes[index].deps.clone();

        if recurse
        {
            for node in to_remove 
            {
                self.inner_remove(node, recurse);
            }
        }

        //remove refs to this node
        for node in &mut self.nodes
        {
            node.parents.retain(|&x| x != index);
            node.deps.retain(|&x| x != index);
        }

        // mark node for deletion
        self.nodes[index].id = self.nodes.len();
    }
    pub fn check_topology(&self) -> Result<(), String>
    {
        let mut g = self.clone();
        let mut out_edges = Vec::<usize>::new();
        let mut orphans = Vec::<usize>::new();
        for node in &g.nodes
        {
            if node.parents.len() == 0
            {
                orphans.push(node.id);
            }
        }
        while let Some(top) = orphans.pop()
        {
            out_edges.push(top.clone());
            let children = g.nodes[top].deps.clone();
            for child in children
            {
                g.unlink(&top, &child).unwrap();
                if g.nodes[child].parents.is_empty()
                {
                    orphans.push(child);
                }
            }
        }
        for node in &g.nodes
        {
            if !node.parents.is_empty() || !node.deps.is_empty()
            {
                return Err("Idiot. Haven't you heard of a DAG before?".to_string());
            }
        }
        return Ok(());
    }
    pub fn link(&mut self, parent: &usize, child: &usize) -> Result<(), String>
    {
        match self.nodes.get(*parent)
        {
            None => { return Err(format!("Parent node {} doesn't exist.", parent).to_string()); },
            _ => {}
        }
        match self.nodes.get(*child)
        {
            None => { return Err(format!("Child node {} doesn't exist.", child).to_string()); },
            _ => {}
        }
        if !self.nodes[*parent].deps.contains(child)
        {
            self.nodes[*parent].deps.push(child.clone());
        }
        if !self.nodes[*child].parents.contains(parent)
        {
            self.nodes[*child].parents.push(parent.clone());
        }
        self.validate()
    }
    pub fn unlink(&mut self, parent: &usize, child: &usize) -> Result<(), String>
    {
        match self.nodes.get(*parent)
        {
            None => { return Err(format!("Parent node {} doesn't exist.", parent).to_string()); },
            _ => {}
        }
        match self.nodes.get(*child)
        {
            None => { return Err(format!("Child node {} doesn't exist.", child).to_string()); },
            _ => {}
        }
        self.nodes[*parent].deps.retain( |x| *x != *child);
        self.nodes[*child].parents.retain( |x| *x != *parent);
        Ok(())
    }
    pub fn show(&self, parent: &usize, mut level: u128, overwhelm: bool)
    {
        match self.nodes[*parent].node_type
        {
            NodeType::Goal => {
                // always print goals
                self.nodes[*parent].print(level);
                level += 1;
            },
            _ => {
                // only print tasks and conditions if they are leaves 
                if overwhelm // nested structure if currently overwhelming the user
                {
                    level += 1
                }
                if overwhelm || self.nodes[*parent].deps.is_empty()
                {
                    self.nodes[*parent].print(level);
                }
            }
        }
        for child in &self.nodes[*parent].deps
        {
            self.show(child, level, overwhelm);
        }

    }
    pub fn relabel(&mut self, id: usize, new_description: String) -> Result<(), String>
    {
        match self.nodes.get(id)
        {
            None => { return Err(format!("Node with id {} doesn't exist.", id).to_string()); },
            _ => {}
        }
        self.nodes[id].description = new_description;
        Ok(())
    }
    pub fn todos(&self, overwhelm: bool)
    {
        for node in &self.nodes
        {
            if node.parents.is_empty() 
            {
                self.show(&node.id, 0, overwhelm);
            }
        }
    }
    pub fn save(&self)
    {
        let mut f = match File::create(&self.location.as_str()) {
            Ok(file) => file,
            Err(_error) => {panic!("Unable to open file")}
        };

        write!(f, "{}", serde_json::to_string(&self).unwrap()).unwrap();
    }
}
