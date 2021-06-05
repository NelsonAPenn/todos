extern crate serde;
extern crate serde_json;


use std::fmt;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, PartialEq)]
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

impl Node
{
    pub fn print(&self, goal_color: &String, condition_color: &String, task_color: &String, level: u128)
    {
        for _i in 0..level
        {
            print!("  ");
        }
        match self.node_type {
            NodeType::Goal => {
                println!("\x1B[{}m{} goal (id {}):\x1B[00m", goal_color, &self.description, &self.id);
            },
            NodeType::Condition => {
                println!("\x1B[{}m{} ({}): {}\x1B[00m", condition_color, &self.id, self.node_type ,&self.description);
            },
            _ => {
                println!("\x1B[{}m{} ({}): {}\x1B[00m", task_color, &self.id, self.node_type ,&self.description);
            }
        }
    }
}

