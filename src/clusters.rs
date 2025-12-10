use itertools::Itertools;
use ordered_float::OrderedFloat;
use std::collections::{HashMap, HashSet};

/// Clusters a list of numbers based on tolerance
///
/// # Arguments
/// * `xs` - A vector of numbers to cluster
/// * `tolerance` - The maximum difference between consecutive elements in a cluster
///
/// # Returns
/// A vector of vectors, where each inner vector represents a cluster
fn cluster_list(
    mut xs: Vec<OrderedFloat<f32>>,
    tolerance: OrderedFloat<f32>,
) -> Vec<Vec<OrderedFloat<f32>>> {
    let zero = OrderedFloat(0.0f32);

    if tolerance == zero || xs.len() < 2 {
        xs.sort();
        return xs.into_iter().map(|x| vec![x]).collect();
    }

    xs.sort();
    let mut groups: Vec<Vec<OrderedFloat<f32>>> = Vec::new();
    let mut current_group = vec![xs[0]];
    let mut last = xs[0];

    for &x in xs.iter().skip(1) {
        if x <= last + tolerance {
            current_group.push(x);
        } else {
            groups.push(current_group);
            current_group = vec![x];
        }
        last = x;
    }

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
fn make_cluster_dict(values: Vec<f32>, tolerance: f32) -> HashMap<OrderedFloat<f32>, usize> {
    let unique_values: Vec<OrderedFloat<f32>> = values
        .into_iter()
        .map(OrderedFloat)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let clusters = cluster_list(unique_values, OrderedFloat(tolerance));

    let mut result = HashMap::new();
    for (cluster_index, cluster) in clusters.into_iter().enumerate() {
        for val in cluster {
            result.insert(val, cluster_index);
        }
    }

    result
}
pub(crate) fn cluster_objects<T, F>(
    xs: Vec<T>,
    key_fn: F,
    tolerance: f32,
    preserve_order: bool,
) -> Vec<Vec<T>>
where
    T: Clone,
    F: Fn(&T) -> f32,
{
    if xs.is_empty() {
        return vec![];
    }

    let values: Vec<f32> = xs.iter().map(&key_fn).collect();
    let cluster_dict = make_cluster_dict(values, tolerance);

    let mut cluster_tuples: Vec<(T, usize)> = xs
        .into_iter()
        .map(|x| {
            let key_value = OrderedFloat(key_fn(&x));
            let cluster_id = cluster_dict.get(&key_value).copied().unwrap_or(0);
            (x, cluster_id)
        })
        .collect();

    if !preserve_order {
        cluster_tuples.sort_by_key(|(_, cluster_id)| *cluster_id);
    }

    cluster_tuples
        .into_iter()
        .group_by(|(_, cluster_id)| *cluster_id)
        .into_iter()
        .map(|(_, group)| group.map(|(item, _)| item).collect())
        .collect()
}
