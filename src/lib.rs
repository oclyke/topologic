use core::hash::Hash;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub enum DependencyError {
    SelfReference,
    CircularDependency,
}

/// A map of direct dependencies.
/// For a given node the value is the set of direct dependencies of that node.
type DirectDependencyMap<T> = HashMap<T, HashSet<T>>;

/// Remove a node from a dependency map.
/// This removes the node from the map and removes the node from the dependency sets of all other nodes.
/// # Arguments
/// * `map` - The dependency map to remove the node from.
/// * `node` - The node to remove from the dependency map.
fn dependency_map_remove_node<T: Eq + Hash>(map: &mut DirectDependencyMap<T>, node: &T) {
    map.remove(node);
    for (_, deps) in &mut *map {
        deps.remove(&node);
    }
    map.retain(|_, deps| deps.len() > 0);
}

/// A directed acyclic graph of dependencies.
/// # Type Parameters
/// * `T` - The type of the nodes in the graph.
/// # Fields
/// * `nodes` - The set of nodes in the graph.
/// * `forward_dependencies` - A map of direct dependencies.
/// * `backward_dependencies` - A map of direct dependents.
/// # Methods
/// * `new()` - Create a new empty graph.
/// * `depend_on()` - Add a dependency between two nodes.
/// * `depends_on()` - Check if one node depends on another.
/// * `get_forward_dependencies()` - Get the set of nodes that a given node depends on.
/// * `get_backward_dependencies()` - Get the set of nodes that depend on a given node.
/// * `get_leaves()` - Get the set of nodes that have no dependencies.
/// * `get_roots()` - Get the set of nodes that have no dependents.
/// * `get_forward_dependency_topological_layers()` - Get the topological layers of the graph in forward direction.
/// * `get_backward_dependency_topological_layers()` - Get the topological layers of the graph in backward direction.
#[derive(Clone)]
pub struct AcyclicDependencyGraph<T> {
    nodes: HashSet<T>,
    forward_dependencies: DirectDependencyMap<T>,
    backward_dependencies: DirectDependencyMap<T>,
}

impl<T> AcyclicDependencyGraph<T>
where
    T: Eq + Hash + Copy,
{
    /// Create a new empty graph.
    /// # Returns
    /// A new empty graph.
    pub fn new() -> Self {
        AcyclicDependencyGraph {
            nodes: HashSet::new(),
            forward_dependencies: HashMap::new(),
            backward_dependencies: HashMap::new(),
        }
    }

    /// Remove a node from the graph.
    /// # Arguments
    /// * `node` - The node to remove from the graph.
    /// # Remarks
    /// This removes the node from the graph and removes the node from the dependency sets of all other nodes.
    fn remove_node(&mut self, node: T) {
        self.nodes.remove(&node);
        dependency_map_remove_node(&mut self.forward_dependencies, &node);
        dependency_map_remove_node(&mut self.backward_dependencies, &node);
    }

    /// Get the set of nodes that have no dependencies.
    /// # Returns
    /// The set of nodes that have no dependencies.
    pub fn get_leaves(&self) -> HashSet<T> {
        let mut leaves: HashSet<T> = HashSet::new();
        for node in &self.nodes {
            if self.forward_dependencies.get(&node).is_none() {
                leaves.insert(*node);
            }
        }
        return leaves;
    }

    /// Get the set of nodes that have no dependents.
    /// # Returns
    /// The set of nodes that have no dependents.
    pub fn get_roots(&self) -> HashSet<T> {
        let mut roots: HashSet<T> = HashSet::new();
        for node in &self.nodes {
            if self.backward_dependencies.get(&node).is_none() {
                roots.insert(*node);
            }
        }
        return roots;
    }

    /// Add a dependency between two nodes.
    /// # Arguments
    /// * `from` - The node that depends on the other node.
    /// * `to` - The node that is depended on.
    /// # Returns
    /// `Ok(())` if the dependency was added successfully.
    /// `Err(DependencyError::SelfReference)` if the dependency would create a self reference.
    /// `Err(DependencyError::CircularDependency)` if the dependency would create a circular dependency.
    pub fn depend_on(&mut self, from: T, to: T) -> Result<(), DependencyError> {
        if from == to {
            return Err(DependencyError::SelfReference);
        }
        if self.depends_on(to, from) {
            return Err(DependencyError::CircularDependency);
        }

        // ensure that nodes are accounted for in the graph
        self.nodes.insert(from);
        self.nodes.insert(to);

        // add the forward and backward dependency edges
        self.forward_dependencies
            .entry(from)
            .or_insert(HashSet::new())
            .insert(to);
        self.backward_dependencies
            .entry(to)
            .or_insert(HashSet::new())
            .insert(from);

        return Ok(());
    }

    /// Check if one node depends on another.
    /// # Arguments
    /// * `source` - The node that depends on the other node.
    /// * `target` - The node that is depended on.
    /// # Returns
    /// `true` if the source node depends on the target node.
    /// `false` if the source node does not depend on the target node.
    pub fn depends_on(&self, source: T, target: T) -> bool {
        self.get_forward_dependencies(source).contains(&target)
    }

    /// Get the set of nodes that a given node depends on.
    /// # Arguments
    /// * `node` - The node to get the dependencies of.
    /// # Returns
    /// The set of nodes that the given node depends on.
    pub fn get_forward_dependencies(&self, node: T) -> HashSet<T> {
        let mut out = HashSet::new();

        let mut discovered = vec![node];
        while discovered.len() > 0 {
            let mut discoveries: Vec<T> = Vec::new();
            for node in discovered {
                // get direct dependencies of the given node
                let direct_dependencies: &HashSet<T> = match self.forward_dependencies.get(&node) {
                    Some(deps) => deps,
                    None => continue,
                };

                // search the direct dependecies for newly discovered dependencies
                for node in direct_dependencies {
                    match out.insert(*node) {
                        true => discoveries.push(*node),
                        false => continue,
                    }
                }
            }
            discovered = discoveries;
        }

        return out;
    }

    /// Get the set of nodes that depend on a given node.
    /// # Arguments
    /// * `node` - The node to get the dependents of.
    /// # Returns
    /// The set of nodes that depend on the given node.
    pub fn get_backward_dependencies(&self, node: T) -> HashSet<T> {
        let mut out = HashSet::new();

        let mut discovered = vec![node];
        while discovered.len() > 0 {
            let mut discoveries: Vec<T> = Vec::new();
            for node in discovered {
                // get direct dependencies of the given node
                let direct_dependencies: &HashSet<T> = match self.backward_dependencies.get(&node) {
                    Some(deps) => deps,
                    None => continue,
                };

                // search the direct dependecies for newly discovered dependencies
                for node in direct_dependencies {
                    match out.insert(*node) {
                        true => discoveries.push(*node),
                        false => continue,
                    }
                }
            }
            discovered = discoveries;
        }

        return out;
    }

    /// Get the topological layers of the graph in forward direction.
    /// # Returns
    /// The topological layers of the graph in forward direction.
    /// Each layer is a set of nodes.
    /// The first layer contains the nodes that have no dependencies.
    /// Each subsequent layer contains the nodes that depend on the nodes in the previous layer.
    /// The last layer contains nodes that depend on all other nodes.
    /// # Remarks
    /// The particular ordering of topological layers is not guaranteed.
    /// The only guarantee is that the nodes in each layer depend only on the nodes in the previous layers.
    pub fn get_forward_dependency_topological_layers(&self) -> Vec<HashSet<T>> {
        let mut layers: Vec<HashSet<T>> = Vec::new();
        let mut shrinking_graph = self.clone();
        loop {
            let leaves = shrinking_graph.get_leaves();
            if leaves.len() == 0 {
                break;
            }
            for leaf in &leaves {
                shrinking_graph.remove_node(*leaf);
            }
            layers.push(leaves);
        }
        return layers;
    }

    /// Get the topological layers of the graph in backward direction.
    /// # Returns
    /// The topological layers of the graph in backward direction.
    /// Each layer is a set of nodes.
    /// The first layer contains the nodes that have no dependents.
    /// Each subsequent layer contains the nodes that the nodes in the previous layer depend on.
    /// The last layer contains nodes that are depended on by all other nodes.
    /// # Remarks
    /// The particular ordering of topological layers is not guaranteed.
    /// The only guarantee is that the nodes in each layer are depended on only by the nodes in the previous layers.
    pub fn get_backward_dependency_topological_layers(&self) -> Vec<HashSet<T>> {
        let mut layers: Vec<HashSet<T>> = Vec::new();
        let mut shrinking_graph = self.clone();
        loop {
            let roots = shrinking_graph.get_roots();
            if roots.len() == 0 {
                break;
            }
            for root in &roots {
                shrinking_graph.remove_node(*root);
            }
            layers.push(roots);
        }
        return layers;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_referential_dependencies_detected() {
        let mut graph = AcyclicDependencyGraph::new();
        assert!(graph.depend_on("a", "a").is_err());
    }

    #[test]
    fn circular_dependencies_detected() {
        let mut graph = AcyclicDependencyGraph::new();
        graph.depend_on("a", "b").unwrap();
        graph.depend_on("b", "c").unwrap();
        assert!(graph.depend_on("c", "a").is_err());
    }

    #[test]
    fn simple_topological_sort_forward() {
        let mut graph = AcyclicDependencyGraph::new();
        graph.depend_on("cake", "eggs").unwrap();
        graph.depend_on("cake", "flour").unwrap();
        graph.depend_on("eggs", "chickens").unwrap();
        graph.depend_on("flour", "grain").unwrap();
        graph.depend_on("chickens", "grain").unwrap();
        graph.depend_on("grain", "soil").unwrap();
        graph.depend_on("grain", "water").unwrap();
        graph.depend_on("chickens", "water").unwrap();

        let layers = graph.get_forward_dependency_topological_layers();
        println!("layers: {:?}", layers);

        // check forward and backward direct dependencies
        // note: we cannot guarantee any particular topological ordering so checking the direct dependencies is a second-best option
        let fwd_bckwd_check = |node: &str, expected_fwd: Vec<&str>, expected_bwd: Vec<&str>| {
            let fwd = graph.get_forward_dependencies(node);
            let bwd = graph.get_backward_dependencies(node);

            assert_eq!(fwd.len(), expected_fwd.len());
            assert_eq!(bwd.len(), expected_bwd.len());
            for expected_node in expected_fwd {
                assert!(fwd.contains(expected_node));
            }
            for expected_node in expected_bwd {
                assert!(bwd.contains(expected_node));
            }
        };
        fwd_bckwd_check(
            "cake",
            vec!["eggs", "grain", "soil", "chickens", "water", "flour"],
            vec![],
        );
        fwd_bckwd_check(
            "eggs",
            vec!["chickens", "grain", "soil", "water"],
            vec!["cake"],
        );
        fwd_bckwd_check("flour", vec!["grain", "soil", "water"], vec!["cake"]);
        fwd_bckwd_check(
            "chickens",
            vec!["grain", "soil", "water"],
            vec!["cake", "eggs"],
        );
        fwd_bckwd_check(
            "grain",
            vec!["water", "soil"],
            vec!["cake", "eggs", "flour", "chickens"],
        );
        fwd_bckwd_check(
            "soil",
            vec![],
            vec!["grain", "eggs", "flour", "cake", "chickens"],
        );
        fwd_bckwd_check(
            "water",
            vec![],
            vec!["grain", "eggs", "flour", "cake", "chickens"],
        );
    }
}
