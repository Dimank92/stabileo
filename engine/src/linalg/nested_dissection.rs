//! Nested Dissection ordering for sparse matrix factorization.
//!
//! Recursively partitions the matrix adjacency graph using vertex separators
//! (George 1973; METIS-style recursive bisection), producing an ordering with
//! minimal fill-in for 2D and 3D structural meshes and building frames.

use super::sparse::CscMatrix;
use std::collections::VecDeque;

/// Compute a fill-reducing Nested Dissection permutation for an SPD matrix.
///
/// Returns `perm` where `perm[new_idx] = old_idx`.
pub fn nested_dissection_order(matrix: &CscMatrix) -> Vec<usize> {
    let n = matrix.n;
    if n <= 1 {
        return (0..n).collect();
    }

    // Build symmetric adjacency list (unweighted undirected graph)
    let mut adj = vec![Vec::new(); n];
    for j in 0..n {
        for k in matrix.col_ptr[j]..matrix.col_ptr[j + 1] {
            let i = matrix.row_idx[k];
            if i != j {
                adj[j].push(i);
                adj[i].push(j);
            }
        }
    }
    for list in &mut adj {
        list.sort_unstable();
        list.dedup();
    }

    let mut perm = Vec::with_capacity(n);
    let all_nodes: Vec<usize> = (0..n).collect();
    recursive_bisection(&adj, &all_nodes, &mut perm);

    perm
}

/// Recursively bisect a subgraph, appending ordered nodes to `perm`.
fn recursive_bisection(adj: &[Vec<usize>], nodes: &[usize], perm: &mut Vec<usize>) {
    let count = nodes.len();
    if count <= 16 {
        // Base case: small subgraph, append nodes directly
        perm.extend_from_slice(nodes);
        return;
    }

    // Subgraph membership lookup
    let max_id = adj.len();
    let mut in_subgraph = vec![false; max_id];
    for &node in nodes {
        in_subgraph[node] = true;
    }

    // 1. Find a pseudo-peripheral node in this subgraph
    let root = find_pseudo_peripheral(adj, nodes[0], &in_subgraph);

    // 2. Compute BFS level sets from root
    let mut dist = vec![-1isize; max_id];
    let mut queue = VecDeque::new();
    queue.push_back(root);
    dist[root] = 0;
    let mut max_dist = 0;

    while let Some(u) = queue.pop_front() {
        let d = dist[u];
        if d > max_dist {
            max_dist = d;
        }
        for &v in &adj[u] {
            if in_subgraph[v] && dist[v] == -1 {
                dist[v] = d + 1;
                queue.push_back(v);
            }
        }
    }

    // If graph is disconnected or max_dist <= 1, no separator needed
    if max_dist <= 1 {
        perm.extend_from_slice(nodes);
        return;
    }

    // 3. Middle level set forms the candidate vertex separator
    let mid = max_dist / 2;
    let mut part_a = Vec::new();
    let mut part_b = Vec::new();
    let mut separator = Vec::new();

    for &node in nodes {
        let d = dist[node];
        if d < mid {
            part_a.push(node);
        } else if d > mid {
            part_b.push(node);
        } else {
            separator.push(node);
        }
    }

    // Ensure neither partition is completely empty to guarantee termination
    if part_a.is_empty() || part_b.is_empty() {
        perm.extend_from_slice(nodes);
        return;
    }

    // 4. Recurse on part A, then part B, and finally place separator at the end
    recursive_bisection(adj, &part_a, perm);
    recursive_bisection(adj, &part_b, perm);
    perm.extend_from_slice(&separator);
}

/// Find a pseudo-peripheral node in the subgraph via successive BFS iterations.
fn find_pseudo_peripheral(adj: &[Vec<usize>], start_node: usize, in_subgraph: &[bool]) -> usize {
    let mut current = start_node;
    let max_id = adj.len();

    for _ in 0..3 {
        let mut dist = vec![-1isize; max_id];
        let mut queue = VecDeque::new();
        queue.push_back(current);
        dist[current] = 0;
        let mut furthest = current;
        let mut max_d = 0;

        while let Some(u) = queue.pop_front() {
            let d = dist[u];
            if d > max_d {
                max_d = d;
                furthest = u;
            }
            for &v in &adj[u] {
                if in_subgraph[v] && dist[v] == -1 {
                    dist[v] = d + 1;
                    queue.push_back(v);
                }
            }
        }

        if furthest == current {
            break;
        }
        current = furthest;
    }

    current
}
