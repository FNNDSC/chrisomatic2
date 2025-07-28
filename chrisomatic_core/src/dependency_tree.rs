pub(crate) use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableDiGraph;
use petgraph::{Direction, acyclic::Acyclic};

pub(crate) type Dag<T> = Acyclic<StableDiGraph<T, ()>>;

// HINT: a directed edge A->B means B depends on A.

/// A directed acyclic graph (DAG) of computational jobs to run.
///
/// `T` should be [Clone] so that it can be retrieved with a separate lifetime from the tree.
pub struct DependencyTree<T>(pub(crate) Dag<T>);

impl<T> DependencyTree<T> {
    /// Count the number of nodes.
    pub fn count(&self) -> usize {
        self.0.node_count()
    }
}

impl<T: Clone> DependencyTree<T> {
    /// Get the "roots" of the DAG, i.e. all nodes which have no dependencies.
    pub(crate) fn start(&self) -> Vec<(NodeIndex, T)> {
        self.0
            .node_indices()
            .filter(|i| self.is_ready(*i))
            .filter_map(|i| self.0.node_weight(i).map(|w| (i, w.clone())))
            .collect()
    }

    /// Returns `true` if the node has no dependencies.
    fn is_ready(&self, n: NodeIndex) -> bool {
        self.0
            .neighbors_directed(n, Direction::Incoming)
            .next()
            .is_none()
    }

    /// Consider the specified node as "done" and remove it. Return all dependent
    /// nodes which are now ready to run because of the removal.
    pub(crate) fn after(&mut self, i: NodeIndex) -> Vec<(NodeIndex, T)> {
        let children: Vec<_> = self.0.neighbors(i).collect();
        self.0.remove_node(i);
        children
            .into_iter()
            .filter_map(|i| -> Option<(NodeIndex, T)> {
                if self.is_ready(i) {
                    self.0.node_weight(i).map(|w| (i, w.clone()))
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use petgraph::data::Build;

    use super::*;

    #[test]
    fn test_dependency_tree() {
        let mut graph: Dag<char> = Acyclic::new();
        let a = graph.add_node('a');
        let b = graph.add_node('b');
        let c = graph.add_node('c');
        let d = graph.add_node('d');
        let e = graph.add_node('e');
        let f = graph.add_node('f');
        /*
         *       a  e
         *      / \  \
         *     b   c  f
         *          \
         *           d
         */
        graph.try_add_edge(a, b, ()).unwrap();
        graph.try_add_edge(a, c, ()).unwrap();
        graph.try_add_edge(c, d, ()).unwrap();
        graph.try_add_edge(e, f, ()).unwrap();

        let mut dep_tree = DependencyTree(graph);
        assert_eq!(dep_tree.count(), 6);

        assert!(dep_tree.is_ready(a));
        assert!(dep_tree.is_ready(e));
        assert!(!dep_tree.is_ready(b));
        assert!(!dep_tree.is_ready(c));
        assert!(!dep_tree.is_ready(d));

        let expected = HashSet::from_iter([(a, 'a'), (e, 'e')]);
        let actual: HashSet<_> = dep_tree.start().into_iter().collect();
        assert_eq!(actual, expected);

        let actual: HashSet<_> = dep_tree.after(a).into_iter().collect();
        let expected = HashSet::from_iter([(b, 'b'), (c, 'c')]);
        assert_eq!(actual, expected);
        assert!(!dep_tree.0.contains_node(a));
        assert!(dep_tree.0.contains_node(b));
        assert!(dep_tree.0.contains_node(c));
        assert!(dep_tree.0.contains_node(d));
        assert!(dep_tree.0.contains_node(e));
        assert!(dep_tree.0.contains_node(f));

        assert_eq!(dep_tree.after(e), vec![(f, 'f')]);
        assert!(!dep_tree.0.contains_node(e));
        assert!(dep_tree.0.contains_node(f));
        assert!(dep_tree.after(f).is_empty());
        assert!(!dep_tree.0.contains_node(f));

        assert!(dep_tree.after(b).is_empty());
        assert_eq!(dep_tree.after(c), vec![(d, 'd')]);
    }
}
