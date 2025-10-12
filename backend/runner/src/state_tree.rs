//! Tree-based state management for Lua execution caching.

use crate::{LuaRunnerError, MAX_CACHE_NODES, Result};
use std::time::{SystemTime, UNIX_EPOCH};

/// Type alias for execution path (sequence of commands).
pub type ExecutionPath = Vec<String>;

/// A node in the state tree representing a single execution state.
#[derive(Debug, Clone)]
pub struct StateNode {
    /// The command that was executed to reach this state.
    pub command: String,

    /// Serialized Lua table state (JSON format).
    pub state: String,

    /// Timestamp when this node was created (Unix timestamp in seconds).
    pub created_at: u64,

    /// Timestamp when this node was last accessed (Unix timestamp in seconds).
    pub last_accessed: u64,

    /// Number of times this node has been accessed.
    pub access_count: usize,

    /// Child nodes (subsequent executions from this state).
    pub children: Vec<StateNode>,
}

impl StateNode {
    /// Creates a new state node.
    pub fn new(command: String, state: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            command,
            state,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            children: Vec::new(),
        }
    }

    /// Updates the last accessed timestamp and increments access count.
    pub fn touch(&mut self) {
        self.last_accessed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.access_count += 1;
    }

    /// Finds a child node by command.
    pub fn find_child(&mut self, command: &str) -> Option<&mut StateNode> {
        self.children
            .iter_mut()
            .find(|child| child.command == command)
    }

    /// Adds a child node.
    pub fn add_child(&mut self, node: StateNode) {
        self.children.push(node);
    }

    /// Counts the total number of nodes in this subtree (including self).
    pub fn count_nodes(&self) -> usize {
        1 + self
            .children
            .iter()
            .map(|child| child.count_nodes())
            .sum::<usize>()
    }

    /// Collects all nodes in this subtree with their access times.
    fn collect_nodes_with_access_time(
        &self,
        nodes: &mut Vec<(u64, Vec<String>)>,
        path: Vec<String>,
    ) {
        let mut current_path = path.clone();
        current_path.push(self.command.clone());
        nodes.push((self.last_accessed, current_path.clone()));

        for child in &self.children {
            child.collect_nodes_with_access_time(nodes, current_path.clone());
        }
    }

    /// Removes a node at the specified path.
    fn remove_node_at_path(&mut self, path: &[String]) -> bool {
        if path.is_empty() {
            return false;
        }

        if path.len() == 1 {
            if let Some(pos) = self
                .children
                .iter()
                .position(|child| child.command == path[0])
            {
                self.children.remove(pos);
                return true;
            }
            return false;
        }

        if let Some(child) = self.find_child(path[0].as_str()) {
            return child.remove_node_at_path(&path[1..]);
        }

        false
    }
}

/// Tree structure for managing execution states.
#[derive(Debug)]
pub struct StateTree {
    /// Root node representing the initial state.
    root: StateNode,

    /// Total number of nodes in the tree.
    node_count: usize,
}

impl StateTree {
    /// Creates a new state tree with an initial root state.
    pub fn new(initial_state: String) -> Self {
        Self {
            root: StateNode::new(String::new(), initial_state),
            node_count: 1,
        }
    }

    /// Finds a node following the given execution path.
    ///
    /// Returns the state of the deepest matching node and the remaining unmatched path.
    /// Touches (updates access time) all nodes along the matched path.
    pub fn find_node(&mut self, path: &ExecutionPath) -> Result<(String, Vec<String>)> {
        if path.is_empty() {
            self.root.touch();
            return Ok((self.root.state.clone(), vec![]));
        }

        self.root.touch();
        let mut current_children = &mut self.root.children;
        let mut depth = 0;
        let mut current_state = self.root.state.clone();

        for command in path.iter() {
            if let Some(pos) = current_children.iter().position(|c| c.command == *command) {
                let child = &mut current_children[pos];
                child.touch();
                current_state = child.state.clone();
                current_children = &mut child.children;
                depth += 1;
            } else {
                return Ok((current_state, path[depth..].to_vec()));
            }
        }

        Ok((current_state, vec![]))
    }

    /// Inserts a new node at the end of the given path.
    pub fn insert_node(
        &mut self,
        path: &[impl AsRef<str>],
        command: String,
        state: String,
    ) -> Result<()> {
        if self.node_count >= MAX_CACHE_NODES {
            self.evict_lru()?;
        }

        let new_node = StateNode::new(command, state);

        if path.is_empty() {
            self.root.add_child(new_node);
            self.root.touch();
            self.node_count += 1;
            return Ok(());
        }

        self.root.touch();
        let mut current_children = &mut self.root.children;

        for parent_command in path.iter() {
            if let Some(pos) = current_children
                .iter()
                .position(|c| c.command == parent_command.as_ref())
            {
                let child = &mut current_children[pos];
                child.touch();
                current_children = &mut child.children;
            } else {
                return Err(LuaRunnerError::InvalidPath(format!(
                    "Parent path not found at depth {}",
                    path.len()
                )));
            }
        }

        current_children.push(new_node);
        self.node_count += 1;

        Ok(())
    }

    /// Evicts the least recently used node from the tree.
    fn evict_lru(&mut self) -> Result<()> {
        let mut nodes = Vec::new();

        for child in &self.root.children {
            child.collect_nodes_with_access_time(&mut nodes, Vec::new());
        }

        if nodes.is_empty() {
            return Err(LuaRunnerError::CacheCapacityExceeded);
        }

        nodes.sort_by_key(|(access_time, _)| *access_time);

        if let Some((_, path)) = nodes.first() {
            if self.root.remove_node_at_path(path.as_slice()) {
                self.node_count = self.root.count_nodes();
                return Ok(());
            }
        }

        Err(LuaRunnerError::CacheCapacityExceeded)
    }

    /// Gets the current number of nodes in the tree.
    pub fn node_count(&self) -> usize {
        self.node_count
    }

    /// Gets a reference to the root node.
    pub fn root(&self) -> &StateNode {
        &self.root
    }

    /// Clears all nodes except the root.
    pub fn clear(&mut self) {
        self.root.children.clear();
        self.node_count = 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EMPTY_COMMANDS: [&str; 0] = [];

    #[test]
    fn test_state_node_creation() {
        let node = StateNode::new("x = 10".to_string(), "{}".to_string());
        assert_eq!(node.command, "x = 10");
        assert_eq!(node.state, "{}");
        assert_eq!(node.access_count, 1);
        assert!(node.children.is_empty());
    }

    #[test]
    fn test_state_node_touch() {
        let mut node = StateNode::new("x = 10".to_string(), "{}".to_string());
        let initial_access = node.last_accessed;
        let initial_count = node.access_count;

        std::thread::sleep(std::time::Duration::from_millis(10));
        node.touch();

        assert!(node.last_accessed >= initial_access);
        assert_eq!(node.access_count, initial_count + 1);
    }

    #[test]
    fn test_state_tree_creation() {
        let tree = StateTree::new("{}".to_string());
        assert_eq!(tree.node_count(), 1);
        assert_eq!(tree.root().state, "{}");
    }

    #[test]
    fn test_insert_and_find_node() {
        let mut tree = StateTree::new("{}".to_string());

        let path1 = vec!["x = 10".to_string()];
        tree.insert_node(
            EMPTY_COMMANDS.as_slice(),
            "x = 10".to_string(),
            r#"{"x": 10}"#.to_string(),
        )
        .unwrap();
        assert_eq!(tree.node_count(), 2);

        let (state, remaining) = tree.find_node(&path1).unwrap();
        assert_eq!(state, r#"{"x": 10}"#);
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_branching_paths() {
        let mut tree = StateTree::new("{}".to_string());

        tree.insert_node(
            EMPTY_COMMANDS.as_slice(),
            "x = 10".to_string(),
            r#"{"x": 10}"#.to_string(),
        )
        .unwrap();

        let path1 = vec!["x = 10".to_string()];
        tree.insert_node(
            EMPTY_COMMANDS.as_slice(),
            "x = 10".to_string(),
            r#"{"x": 10}"#.to_string(),
        )
        .unwrap();

        tree.insert_node(
            &path1,
            "y = 20".to_string(),
            r#"{"x": 10, "y": 20}"#.to_string(),
        )
        .unwrap();
        tree.insert_node(
            &path1,
            "z = 30".to_string(),
            r#"{"x": 10, "z": 30}"#.to_string(),
        )
        .unwrap();

        assert_eq!(tree.node_count(), 4);

        let path2 = vec!["x = 10".to_string(), "y = 20".to_string()];
        let (state, remaining) = tree.find_node(&path2).unwrap();
        assert_eq!(state, r#"{"x": 10, "y": 20}"#);
        assert!(remaining.is_empty());

        let path3 = vec!["x = 10".to_string(), "z = 30".to_string()];
        let (state, remaining) = tree.find_node(&path3).unwrap();
        assert_eq!(state, r#"{"x": 10, "z": 30}"#);
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_partial_path_match() {
        let mut tree = StateTree::new("{}".to_string());
        tree.insert_node(
            EMPTY_COMMANDS.as_slice(),
            "x = 10".to_string(),
            r#"{"x": 10}"#.to_string(),
        )
        .unwrap();

        let path = vec!["x = 10".to_string(), "y = 20".to_string()];
        let (state, remaining) = tree.find_node(&path).unwrap();

        assert_eq!(state, r#"{"x": 10}"#);
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0], "y = 20");
    }

    #[test]
    fn test_clear() {
        let mut tree = StateTree::new("{}".to_string());
        tree.insert_node(
            EMPTY_COMMANDS.as_slice(),
            "x = 10".to_string(),
            r#"{"x": 10}"#.to_string(),
        )
        .unwrap();
        tree.insert_node(
            EMPTY_COMMANDS.as_slice(),
            "y = 20".to_string(),
            r#"{"y": 20}"#.to_string(),
        )
        .unwrap();

        assert_eq!(tree.node_count(), 3);

        tree.clear();
        assert_eq!(tree.node_count(), 1);
        assert!(tree.root().children.is_empty());
    }
}
