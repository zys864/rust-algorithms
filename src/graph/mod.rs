pub mod flow;
pub mod connectivity;

// Represents a union of disjoint sets. Each set's elements are arranged in a
// tree, whose root is the set's representative.
pub struct DisjointSets {
    parent: Vec<usize>
}

impl DisjointSets {
    // Initialize disjoint sets containing one element each.
    pub fn new(size: usize) -> DisjointSets {
        DisjointSets { parent: (0..size).collect() }
    }
    
    // Find the set's representative. Do path compression along the way to make
    // future queries faster.
    pub fn find(&mut self, u: usize) -> usize {
        let pu = self.parent[u];
        if pu != u { self.parent[u] = self.find(pu); }
        self.parent[u]
    }
    
    // Merge the sets containing u and v into a single set containing their
    // union. Returns true if u and v were previously in different sets.
    pub fn merge(&mut self, u: usize, v: usize) -> bool {
        let (pu, pv) = (self.find(u), self.find(v));
        self.parent[pu] = pv;
        pu != pv
    }
}

// A compact graph representation.
pub struct Graph {
    pub first: Vec<Option<usize>>,
    pub next: Vec<Option<usize>>,
    pub endp: Vec<usize>,
}

impl Graph {
    // Initialize a graph with vmax vertices and no edges. For best efficiency,
    // emax should be a tight upper bound on the number of edges to insert.
    pub fn new(vmax: usize, emax: usize) -> Graph {
        Graph {
            first: vec![None; vmax],
            next: Vec::with_capacity(emax),
            endp: Vec::with_capacity(emax)
        }
    }
    
    // Utilities to compute the number of vertices and edges.
    pub fn num_v(&self) -> usize { self.first.len() }
    pub fn num_e(&self) -> usize { self.next.len() }
    
    // Add a directed edge from u to v.
    pub fn add_edge(&mut self, u: usize, v: usize) {
        self.next.push(self.first[u]);
        self.first[u] = Some(self.endp.len());
        self.endp.push(v);
    }
    
    // An undirected edge is two directed edges. If edges are added only via
    // this funcion, the reverse of any edge e can be found at e^1.
    pub fn add_undirected_edge(&mut self, u: usize, v: usize) {
        self.add_edge(u, v);
        self.add_edge(v, u);
    }
    
    // If we think of each even-numbered vertex as a variable, and its successor
    // as its negation, then we can build the implication graph corresponding
    // to any 2-CNF formula. Note that u||v == !u -> v == !v -> u.
    pub fn add_two_sat_clause(&mut self, u: usize, v: usize) {
        self.add_edge(u^1, v);
        self.add_edge(v^1, u);
    }
    
    // Gets vertex u's adjacency list.
    pub fn adj_list<'a>(&'a self, u: usize) -> AdjListIterator<'a> {
        AdjListIterator {
            graph: self,
            next_e: self.first[u]
        }
    }
    
    // Finds the sequence of edges in an Euler path starting from u, assuming it
    // exists and that the graph is directed. To extend this to undirected
    // graphs, keep track of a visited array to skip the reverse edge.
    pub fn euler_path(&self, u: usize) -> Vec<usize> {
        let mut adj_iters = (0..self.num_v()).map(|u| self.adj_list(u))
                            .collect::<Vec<_>>();
        let mut edges = Vec::with_capacity(self.num_e());
        self.euler_recurse(u, &mut adj_iters, &mut edges);
        edges.reverse();
        edges
    }
    
    // Helper function used by euler_path. Note that we can't consume the
    // adjacency list in a for loop because recursive calls may need it.
    fn euler_recurse(&self, u: usize, adj: &mut [AdjListIterator], edges: &mut Vec<usize>) {
        while let Some((e, v)) = adj[u].next() {
            self.euler_recurse(v, adj, edges);
            edges.push(e);
        }
    }
    
    // Kruskal's minimum spanning tree algorithm on an undirected graph.
    pub fn min_spanning_tree(&self, weights: &[i64]) -> Vec<usize> {
        assert_eq!(self.num_e(), 2 * weights.len());
        let mut edges = (0..weights.len()).collect::<Vec<_>>();
        edges.sort_by_key(|&e| weights[e]);
        
        let mut components = DisjointSets::new(self.num_v());
        edges.into_iter()
            .filter(|&e| components.merge(self.endp[2*e], self.endp[2*e+1]))
            .collect()
    }
}

// An iterator for convenient adjacency list traversal.
pub struct AdjListIterator<'a> {
    graph: &'a Graph,
    next_e: Option<usize>
}

impl<'a> Iterator for AdjListIterator<'a> {
    type Item = (usize, usize);
    
    // Produces an outgoing edge and vertex.
    fn next(&mut self) -> Option<Self::Item> {
        self.next_e.map( |e| {
            let v = self.graph.endp[e];
            self.next_e = self.graph.next[e];
            (e, v)
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn test_euler()
    {
        let mut graph = Graph::new(3, 4);
        graph.add_edge(0, 1);
        graph.add_edge(1, 0);
        graph.add_edge(1, 2);
        graph.add_edge(2, 1);
        assert_eq!(graph.euler_path(0), vec![0, 2, 3, 1]);
    }
    
    #[test]
    fn test_min_spanning_tree()
    {
        let mut graph = Graph::new(3, 3);
        graph.add_undirected_edge(0, 1);
        graph.add_undirected_edge(1, 2);
        graph.add_undirected_edge(2, 0);
        let weights = [7, 3, 5];
        assert_eq!(graph.min_spanning_tree(&weights), vec![1, 2]);
    }
}