extern crate num;

use std::fmt;
use std::result::Result as Result;
use std::io::prelude::*;
use std::fs::File;
use num_derive::FromPrimitive;    
use num_traits::FromPrimitive;
use std::io::BufReader;

#[derive(Clone, FromPrimitive, ToPrimitive)]
pub enum NodeType
{
    TASK,
    CONDITION,
    GOAL
}
impl fmt::Display for NodeType
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}", match self {
            NodeType::TASK => "task",
            NodeType::CONDITION => "condition",
            NodeType::GOAL => "goal"
        })
    }
}
impl NodeType
{
    pub fn from_string(s: &String) -> Option<NodeType>
    {
        match &s[..]
        {
            "task" => Some(NodeType::TASK),
            "condition" => Some(NodeType::CONDITION),
            "goal" => Some(NodeType::GOAL),
            _ => None
        }
    }
}

#[derive(Clone)]
pub struct Node
{
    pub id: usize,
    pub description: String,
    pub node_type: NodeType,
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
            NodeType::GOAL => {
                println!("{} goal (id {}):", &self.description, &self.id);
            },
            _ => {
                println!("{} ({}): {}", &self.id, self.node_type ,&self.description);
            }
        }
    }
}

#[derive(Clone)]
pub struct Graph
{
    pub nodes: Vec<Node>
}

#[allow(dead_code)]
impl Graph
{
    pub fn new() -> Graph
    {
        let mut nodes:Vec<Node> = Vec::<Node>::new();
        let f = match File::open("/home/nelson/.todos") {
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
                deps: deps,
                parents: parents
            });

        }
        let graph = Graph {
            nodes: nodes
        };
        match graph.validate()
        {
            Ok(()) => {},
            Err(()) => {panic!("Invalid node list")}
        };
        graph
    }
    pub fn validate(&self) -> Result<(), ()>
    {
        // check indices/ ids are all within bounds
        for i in 0..self.nodes.len()
        {
            // id should be the index of the node
            if self.nodes[i].id != i
            {
                return Err(());
            }
            // indices should all be within bounds
            match self.validate_node(&self.nodes[i])
            {
                Err(()) => { return Err(()); },
                _ => {}
            }
            //links should be double sided
            for p in &self.nodes[i].parents
            {
                if !self.nodes[*p].deps.contains(&i)
                {
                    return Err(());
                }
            }
            for p in &self.nodes[i].deps
            {
                if !self.nodes[*p].parents.contains(&i)
                {
                    return Err(());
                }
            }
        }
        match self.check_topology()
        {
            Err(()) => { return Err(()) },
            _ => {}
        }
        Ok(())
    }
    fn validate_node(&self, n: &Node) -> Result<(), ()>
    {
        for j in &n.deps
        {
            match self.nodes.get(*j)
            {
                None => { return Err(()); },
                _ => {}
            }
        }
        for j in &n.parents
        {
            match self.nodes.get(*j)
            {
                None => { return Err(()); },
                _ => {}
            }
        }
        return Ok(());
    }
    pub fn add_node(&mut self, n: Node) -> Result<usize, ()>
    {
        let mut id_to_return: usize = 0;
        // if it don't work, don't panic
        match self.validate_node(&n)
        {
            Err(()) => {return Err(());},
            Ok(()) => {
                let node_to_add = Node {
                    id: self.nodes.len(),
                    description: n.description,
                    node_type: n.node_type,
                    deps: n.deps,
                    parents: n.parents
                };
                id_to_return = node_to_add.id.clone();
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
            Err(()) => { self.remove_node(self.nodes.len() - 1).unwrap(); } // try to remove last node
        }
        // but if it messes up your graph, then panic
        match self.validate()
        {
            Ok(()) => { return Ok(id_to_return); },
            Err(()) => { panic!("Added crappy node and couldn't get rid of it!!!"); } 
        }
    }
    pub fn remove_node(&mut self, index: usize) -> Result<(), ()>
    {
        match self.nodes.get(index)
        {
            None => { return Err(()); },
            _ => {}
        }

        let to_remove:Vec<usize> = self.nodes[index].deps.clone();
        for node in to_remove 
        {
            self.remove_node(node).unwrap();
        }

        //remove refs to this node
        for node in &mut self.nodes
        {
            node.parents.retain(|&x| x != index);
            node.deps.retain(|&x| x != index);
        }

        //remove node
        self.nodes.retain(|x| x.id != index);
        //update ids/indices
        for node in &mut self.nodes
        {
            if node.id >= index
            {
                node.id -= 1;
            }
            node.parents = node.parents.iter().map(|x| { if x >= &index { x.clone() - 1 } else { x.clone() } }).collect();
            node.deps = node.deps.iter().map(|x| { if x >= &index { x.clone() - 1 } else { x.clone() } }).collect();
        }
        // but if it messes up your graph, then panic
        match self.validate()
        {
            Ok(()) => { return Ok(()); },
            Err(()) => { panic!("Removed node quite badly!!!"); } 
        }
    }
    pub fn check_topology(&self) -> Result<(), ()>
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
                return Err(());
            }
        }
        return Ok(());
    }
    pub fn link(&mut self, parent: &usize, child: &usize) -> Result<(), ()>
    {
        match self.nodes.get(*parent)
        {
            None => { return Err(()); },
            _ => {}
        }
        match self.nodes.get(*child)
        {
            None => { return Err(()); },
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
    pub fn unlink(&mut self, parent: &usize, child: &usize) -> Result<(), ()>
    {
        match self.nodes.get(*parent)
        {
            None => { return Err(()); },
            _ => {}
        }
        match self.nodes.get(*child)
        {
            None => { return Err(()); },
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
            NodeType::GOAL => {
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
    pub fn write(&self)
    {
        let mut f = match File::create("/home/nelson/.todos") {
            Ok(file) => file,
            Err(_error) => {panic!("Unable to open file")}
        };
        let mut all_text = String::new();
        for node in &self.nodes
        {
            all_text += &format!("{}\n", node)[..];
        }
        f.write_all(all_text.as_bytes()).unwrap();
    }
}
