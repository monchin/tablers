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

/// Creates a dictionary mapping values to their cluster indices.
///
/// # Arguments
///
/// * `values` - A vector of values to cluster.
/// * `tolerance` - The tolerance value for clustering.
///
/// # Returns
///
/// A HashMap mapping each value to its cluster index.
fn make_cluster_dict(
    values: Vec<OrderedFloat<f32>>,
    tolerance: OrderedFloat<f32>,
) -> HashMap<OrderedFloat<f32>, usize> {
    let unique_values: Vec<OrderedFloat<f32>> = values
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let clusters = cluster_list(unique_values, tolerance);

    let mut result = HashMap::new();
    for (cluster_index, cluster) in clusters.into_iter().enumerate() {
        for val in cluster {
            result.insert(val, cluster_index);
        }
    }

    result
}
/// Clusters objects based on a key function and tolerance.
///
/// Groups objects together if their key values are within the specified tolerance.
///
/// # Arguments
///
/// * `xs` - The objects to cluster.
/// * `key_fn` - A function that extracts a numeric key from each object.
/// * `tolerance` - The maximum difference for objects to be in the same cluster.
///
/// # Returns
///
/// A vector of vectors, where each inner vector is a cluster of objects.
///
/// # Type Parameters
///
/// * `T` - The type of objects being clustered (must implement Clone).
/// * `F` - The key extraction function type.
pub(crate) fn cluster_objects<T, F>(
    xs: &[T],
    key_fn: F,
    tolerance: OrderedFloat<f32>,
) -> Vec<Vec<T>>
where
    T: Clone,
    F: Fn(&T) -> OrderedFloat<f32>,
{
    if xs.is_empty() {
        return vec![];
    }

    let values: Vec<OrderedFloat<f32>> = xs.iter().map(&key_fn).collect();
    let cluster_dict = make_cluster_dict(values, tolerance);

    let mut cluster_tuples: Vec<(T, usize)> = xs
        .iter()
        .map(|x| {
            let key_value = OrderedFloat(key_fn(x));
            let cluster_id = cluster_dict.get(&key_value).copied().unwrap_or(0);
            (x.clone(), cluster_id)
        })
        .collect();
    cluster_tuples.sort_by_key(|(_, cluster_id)| *cluster_id);

    cluster_tuples
        .into_iter()
        .chunk_by(|(_, cluster_id)| *cluster_id)
        .into_iter()
        .map(|(_, group)| group.map(|(item, _)| item).collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ordered_float::OrderedFloat;

    #[test]
    fn test_cluster_list() {
        let a: Vec<OrderedFloat<f32>> = vec![1.0, 2.0, 3.0, 4.0]
            .into_iter()
            .map(OrderedFloat)
            .collect();
        let expected: Vec<Vec<OrderedFloat<f32>>> = a.iter().map(|&x| vec![x]).collect();
        assert_eq!(cluster_list(a.clone(), OrderedFloat(0.0)), expected);

        let a: Vec<OrderedFloat<f32>> = vec![1.0, 2.0, 3.0, 4.0]
            .into_iter()
            .map(OrderedFloat)
            .collect();
        assert_eq!(cluster_list(a.clone(), OrderedFloat(1.0)), vec![a]);

        let a: Vec<OrderedFloat<f32>> = vec![1.0, 2.0, 5.0, 6.0]
            .into_iter()
            .map(OrderedFloat)
            .collect();
        let expected: Vec<Vec<OrderedFloat<f32>>> = vec![
            vec![OrderedFloat(1.0), OrderedFloat(2.0)],
            vec![OrderedFloat(5.0), OrderedFloat(6.0)],
        ];
        assert_eq!(cluster_list(a, OrderedFloat(1.0)), expected);
    }

    #[test]
    fn test_cluster_objects() {
        let a: Vec<String> = vec!["a", "ab", "abc", "b"]
            .into_iter()
            .map(String::from)
            .collect();

        let result = cluster_objects(
            &a,
            |s: &String| OrderedFloat(s.len() as f32),
            OrderedFloat(0.0),
        );

        assert_eq!(
            result,
            vec![
                vec!["a".to_string(), "b".to_string()],
                vec!["ab".to_string()],
                vec!["abc".to_string()],
            ]
        );

        #[derive(Debug, Clone, PartialEq)]
        struct Item {
            x: f32,
            label: String,
        }

        let b = vec![
            Item {
                x: 1.0,
                label: "a".to_string(),
            },
            Item {
                x: 1.0,
                label: "b".to_string(),
            },
            Item {
                x: 2.0,
                label: "b".to_string(),
            },
            Item {
                x: 2.0,
                label: "b".to_string(),
            },
        ];

        let result = cluster_objects(&b, |item: &Item| OrderedFloat(item.x), OrderedFloat(0.0));
        assert_eq!(
            result,
            vec![
                vec![b[0].clone(), b[1].clone()],
                vec![b[2].clone(), b[3].clone()],
            ]
        );

        let result = cluster_objects(
            &b,
            |item: &Item| match item.label.as_str() {
                "a" => OrderedFloat(1.0),
                "b" => OrderedFloat(2.0),
                _ => OrderedFloat(0.0),
            },
            OrderedFloat(0.0),
        );
        assert_eq!(
            result,
            vec![
                vec![b[0].clone()],
                vec![b[1].clone(), b[2].clone(), b[3].clone()],
            ]
        );
    }
}
