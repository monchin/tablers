use std::collections::{HashMap, HashSet};
use std::hash::Hash;


/// Clusters a list of numbers based on tolerance
/// 
/// # Arguments
/// * `xs` - A vector of numbers to cluster
/// * `tolerance` - The maximum difference between consecutive elements in a cluster
/// 
/// # Returns
/// A vector of vectors, where each inner vector represents a cluster
pub fn cluster_list<T>(mut xs: Vec<T>, tolerance: T) -> Vec<Vec<T>>
where
    T: PartialOrd + Copy + std::ops::Add<Output = T> + PartialEq,
{
    if tolerance == T::default() || xs.len() < 2 {
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        return xs.into_iter().map(|x| vec![x]).collect();
    }

    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let mut groups: Vec<Vec<T>> = Vec::new();
    let mut current_group: Vec<T> = vec![xs[0]];
    let mut last = xs[0];
    
    for &x in &xs[1..] {
        if x <= last + tolerance {
            current_group.push(x);
        } else {
            groups.push(current_group);
            current_group = vec![x];
        }
        last = x;
    }
    
    // 添加最后一个群组
    groups.push(current_group);
    groups
}



/// Creates a dictionary mapping values to their cluster index
/// 
/// # Arguments
/// * `values` - An iterable collection of values to cluster
/// * `tolerance` - The tolerance value for clustering
/// 
/// # Returns
/// A HashMap mapping each value to its cluster index
pub fn make_cluster_dict<T>(values: Vec<T>, tolerance: T) -> HashMap<T, usize>
where
    T: Copy + Eq + Hash + PartialOrd + std::ops::Add<Output = T> + PartialEq,
{
    // Convert to HashSet to remove duplicates, then to Vec for processing
    let unique_values: Vec<T> = values.into_iter().collect::<HashSet<_>>().into_iter().collect();
    
    // Cluster the unique values
    let clusters = cluster_list(unique_values, tolerance);
    
    // Create the dictionary mapping each value to its cluster index
    let mut result = HashMap::new();
    
    for (i, value_cluster) in clusters.into_iter().enumerate() {
        for val in value_cluster {
            result.insert(val, i);
        }
    }
    
    result
}