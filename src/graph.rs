extern crate serde;
extern crate serde_json;

use crate::node::*;
use crate::config::Config;

use std::fs;
use std::result::Result;
use std::io::prelude::*;
use std::fs::File;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;


#[derive(Clone, Serialize, Deserialize)]
pub struct Graph
{
    effective_root: Option<usize>,
    nodes: Vec<Node>,

    #[serde(skip_serializing, skip_deserializing)]
    todos_file: PathBuf,

    #[serde(skip_serializing, skip_deserializing)]
    config: Config 
}

#[allow(dead_code)]
impl Graph
{
    pub fn load(todos_file: PathBuf, config: Config) ->  Graph
    {
        let mut graph: Graph = serde_json::from_str(
            fs::read_to_string(&todos_file)
                .unwrap()
                .as_str()
            ).unwrap();

        graph.config = config;
        graph.todos_file = todos_file;

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

    pub fn add_node_to(&mut self, description: String, node_type: NodeType, to: Option<usize>) -> Result<usize, String>
    {
        let n = Node
        {
            id:0,
            description: description,
            node_type: node_type,
            due_date: None,
            deps: Vec::<usize>::new(),
            parents: match to {
                Some(val) => vec![val],
                None => {
                    if let Some(effective_root) = self.effective_root
                    {
                        vec![effective_root]
                    }
                    else
                    {
                        Vec::<usize>::new()
                    }
                }
            }
        };

        self.add_node(n)
    }

    pub fn add_node_above(&mut self, description: String, node_type: NodeType, above: usize) -> Result<usize, String>
    {
        let node_to_shift = &self.nodes[above];
        let parents = node_to_shift.parents.clone();

        // unlink all references above node 'above'
        for parent in &parents
        {
            match self.unlink(&parent, &above){
                Ok(()) => {},
                Err(message) => {
                    panic!("{}", message);
                }
            }
        }

        let n = Node
        {
            id:0,
            description: description,
            node_type: node_type,
            due_date: None,
            deps: vec![above],
            parents: parents
        };

        let id = self.add_node(n)?;
        self.link(&id, &above).map(|_x| id)
    }


    fn add_node(&mut self, n: Node) -> Result<usize, String>
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
                    Err(message) => panic!("{}", message)
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
        self.batch_remove(vec![index], recurse)
    }

    pub fn batch_remove(&mut self, indices: Vec<usize>, recurse: bool) -> Result<(), String>
    {
        let mut error = None;

        for index in indices.iter()
        {
            match self.nodes.get(*index)
            {
                None => {
                    let new_error_text = format!("Node with id {} not present in todos.\n", index);
                    if let Some(msg) = error
                    {
                        error = Some(msg + &new_error_text[..]);
                    }
                    else
                    {
                        error = Some(new_error_text);
                    }
                },
                _ => { self.inner_remove(index.clone(), recurse); }
            }

        }

        let invalid_val = self.nodes.len().clone();

        //remove invalid nodes
        self.nodes.retain(|x| x.id != invalid_val);

        //update ids/indices
        for i in 0..self.nodes.len()
        {
            self.rename_node(&self.nodes[i].id.clone(), &i);
        }


        // but if it messes up your graph, then panic
        if let Err(message) = self.validate()
        {
            panic!("{}", message);
        }

        if let Some(msg) = error
        {
            Err(msg)
        }
        else
        {
            Ok(())
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
                return Err("Idiot. Haven't you heard of a DAG above?".to_string());
            }
        }

        Ok(())
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

    pub fn todos(&self, overwhelm: bool)
    {
        if let Some(root) = self.effective_root
        {
            self.show(&root, 0, overwhelm, Some(root)).unwrap();
        }
        else
        {
            for node in &self.nodes
            {
                if node.parents.is_empty() 
                {
                    self.show(&node.id, 0, overwhelm, None).unwrap();
                }
            }
        }
    }

    pub fn set_effective_root(&mut self, node_id: Option<usize>) -> Result<(), String>
    {
        if let Some(ref id) = node_id
        {
            if let None = self.nodes.get(*id)
            {
                return Err(format!("Node with id {} not present in todos.", id));
            }
        }

        self.effective_root = node_id;
        Ok(())
    }

    pub fn show(&self, parent: &usize, mut level: u128, overwhelm: bool, started_from: Option<usize>) -> Result<(), String>
    {
        let node = self.nodes.get(*parent).ok_or( format!("Node with id {} not present in todos.", parent))?;

        if 
            overwhelm || // print everything if overwhelming the user
            node.node_type == NodeType::Goal || // always print goals
            node.deps.is_empty() // always print leaves
        {
            node.print(
                &self.config.goal_color,
                &self.config.condition_color,
                &self.config.task_color,
                level);

            // if the node was printed, then we increase the indentation for the children
            level += 1;
        }

        if
            self.config.hide_backlog_items &&
            node.node_type == NodeType::Goal && 
            node.description == self.config.backlog_name &&
            started_from != Some(*parent) 
        {
            // if this is a backlog node and we did not start at this node,
            // then hide its children
            return Ok(())
        }

        for child in &node.deps
        {
            self.show(child, level, overwhelm, started_from).unwrap();
        }
        
        Ok(())

    }
    pub fn relabel(&mut self, id: usize, new_description: String) -> Result<(), String>
    {
        let node = self.nodes.get_mut(id).ok_or( format!("Node with id {} not present in todos.", id))?;
        node.description = new_description;
        Ok(())
    }
    pub fn save(&self)
    {
        let mut f = match File::create(&self.todos_file) {
            Ok(file) => file,
            Err(_error) => {panic!("Unable to open file")}
        };

        write!(f, "{}", serde_json::to_string(&self).unwrap()).unwrap();
    }

    pub fn print_node(&self, id: usize, level: u128) -> Result<(), String>
    {
        let node = self.nodes.get(id).ok_or(
            format!("Node {} not found.", id)
        )?;

        node.print(
            &self.config.goal_color,
            &self.config.condition_color,
            &self.config.task_color,
            level
        );

        Ok(())
    }
}
