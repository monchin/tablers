use crate::edges::*;
use ordered_float::OrderedFloat;
use pdfium_render::prelude::*;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
type Point = (OrderedFloat<f32>, OrderedFloat<f32>);

static DEFAULT_SNAP_TOLERANCE: f32 = 3.0;
static DEFAULT_JOIN_TOLERANCE: f32 = 3.0;
static DEFAULT_INTERSECTION_TOLERANCE: f32 = 3.0;
// use crate::edges::*;
struct Cell {
    text: String,
    bbox: BboxKey,
}
struct Table {
    cells: Vec<Cell>,
    bbox: BboxKey,
    page_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StrategyType {
    Lines,
    LinesStrict,
    Text,
}

struct TfSettings {
    vertiacl_strategy: StrategyType,
    horizontal_strategy: StrategyType,
    snap_x_tolerance: OrderedFloat<f32>,
    snap_y_tolerance: OrderedFloat<f32>,
    join_x_tolerance: OrderedFloat<f32>,
    join_y_tolerance: OrderedFloat<f32>,
    edge_min_length: OrderedFloat<f32>,
    edge_min_length_prefilter: OrderedFloat<f32>,
    intersection_x_tolerance: OrderedFloat<f32>,
    intersection_y_tolerance: OrderedFloat<f32>,
}
impl Default for TfSettings {
    fn default() -> Self {
        TfSettings {
            vertiacl_strategy: StrategyType::Lines,
            horizontal_strategy: StrategyType::Lines,
            snap_x_tolerance: OrderedFloat::from(DEFAULT_SNAP_TOLERANCE),
            snap_y_tolerance: OrderedFloat::from(DEFAULT_SNAP_TOLERANCE),
            join_x_tolerance: OrderedFloat::from(DEFAULT_JOIN_TOLERANCE),
            join_y_tolerance: OrderedFloat::from(DEFAULT_JOIN_TOLERANCE),
            edge_min_length: OrderedFloat::from(3.0),
            edge_min_length_prefilter: OrderedFloat::from(1.0),
            intersection_x_tolerance: OrderedFloat::from(DEFAULT_INTERSECTION_TOLERANCE),
            intersection_y_tolerance: OrderedFloat::from(DEFAULT_INTERSECTION_TOLERANCE),
        }
    }
}

fn filter_edges_by_min_len(edges: &mut Vec<Edge>, min_len: OrderedFloat<f32>) {
    edges.retain(|edge| match edge.edge_type {
        EdgeType::HorizontalLine => (edge.x2 - edge.x1) >= min_len,
        EdgeType::HorizontalRect => (edge.x2 - edge.x1) >= min_len,
        EdgeType::VerticalLine => (edge.y2 - edge.y1) >= min_len,
        EdgeType::VerticalRect => (edge.y2 - edge.y1) >= min_len,
    });
}

fn edges_to_intersections(
    edges: &mut HashMap<Orientation, Vec<Edge>>,
    intersection_x_tolerance: OrderedFloat<f32>,
    intersection_y_tolerance: OrderedFloat<f32>,
) -> HashMap<Point, HashMap<Orientation, Vec<Edge>>> {
    let mut intersections: HashMap<Point, HashMap<Orientation, Vec<Edge>>> = HashMap::new();

    edges
        .get_mut(&Orientation::Vertical)
        .unwrap()
        .sort_by_key(|e| (e.x1, e.y1));
    edges
        .get_mut(&Orientation::Horizontal)
        .unwrap()
        .sort_by_key(|e| (e.y1, e.x1));

    let v_edges = edges.get(&Orientation::Vertical).unwrap();
    let h_edges = edges.get(&Orientation::Horizontal).unwrap();

    for v in v_edges.iter() {
        for h in h_edges.iter() {
            if v.y1 <= h.y1 + intersection_y_tolerance
                && v.y2 >= h.y1 - intersection_y_tolerance
                && v.x1 >= h.x1 - intersection_x_tolerance
                && v.x1 <= h.x2 + intersection_x_tolerance
            {
                let vertex = (v.x1, h.y1);

                let intersection = intersections.entry(vertex).or_default();
                intersection
                    .entry(Orientation::Vertical)
                    .or_default()
                    .push((*v).clone());
                intersection
                    .entry(Orientation::Horizontal)
                    .or_default()
                    .push((*v).clone());
            }
        }
    }
    intersections
}

#[inline]
fn edges_to_set(edges: &[Edge]) -> HashSet<BboxKey> {
    edges.iter().map(|e| e.to_bbox_key()).collect()
}

fn intersections_to_cells(
    intersections: HashMap<Point, HashMap<Orientation, Vec<Edge>>>,
) -> Vec<BboxKey> {
    let edge_connects = |p1: &Point, p2: &Point| -> bool {
        let inter1 = match intersections.get(p1) {
            Some(i) => i,
            None => return false,
        };
        let inter2 = match intersections.get(p2) {
            Some(i) => i,
            None => return false,
        };

        if p1.0 == p2.0 {
            let set1 = edges_to_set(&inter1.get(&Orientation::Vertical).unwrap());
            let set2 = edges_to_set(&inter2.get(&Orientation::Vertical).unwrap());
            if !set1.is_disjoint(&set2) {
                return true;
            }
        }

        if p1.1 == p2.1 {
            let set1 = edges_to_set(&inter1.get(&Orientation::Horizontal).unwrap());
            let set2 = edges_to_set(&inter1.get(&Orientation::Horizontal).unwrap());
            if !set1.is_disjoint(&set2) {
                return true;
            }
        }

        false
    };

    let mut points: Vec<Point> = intersections.keys().cloned().collect();
    points.sort();
    let n_points = points.len();

    let find_smallest_cell = |i: usize| -> Option<BboxKey> {
        if i == n_points - 1 {
            return None;
        }

        let pt1 = &points[i];
        let rest = &points[i + 1..];

        let v_after: Vec<&Point> = rest.iter().filter(|x| x.0 == pt1.0).collect();
        let h_after: Vec<&Point> = rest.iter().filter(|x| x.1 == pt1.1).collect();

        for v_after_pt in &v_after {
            if !edge_connects(pt1, v_after_pt) {
                continue;
            }

            for h_after_pt in &h_after {
                if !edge_connects(pt1, h_after_pt) {
                    continue;
                }

                let pt2: Point = (h_after_pt.0, v_after_pt.1);

                if intersections.contains_key(&pt2)
                    && edge_connects(&pt2, h_after_pt)
                    && edge_connects(&pt2, v_after_pt)
                {
                    return Some((pt1.0, pt1.1, pt2.0, pt2.1));
                }
            }
        }

        None
    };

    (0..n_points)
        .filter_map(|i| find_smallest_cell(i))
        .collect()
}

fn bbox_to_corners(bbox: &BboxKey) -> [Point; 4] {
    let (x1, y1, x2, y2) = *bbox;
    [(x1, y1), (x1, y2), (x2, y1), (x2, y2)]
}

pub fn cells_to_tables(cells: &Vec<BboxKey>) -> Vec<Vec<BboxKey>> {
    let n = cells.len();
    let mut used = vec![false; n];
    let mut tables: Vec<Vec<BboxKey>> = Vec::new();
    let mut current_corners: HashSet<Point> = HashSet::new();
    let mut current_cells: Vec<BboxKey> = Vec::new();

    loop {
        let initial_count = current_cells.len();

        for (i, cell) in cells.iter().enumerate() {
            if used[i] {
                continue;
            }

            let cell_corners = bbox_to_corners(cell);

            if current_cells.is_empty() {
                current_corners.extend(cell_corners);
                current_cells.push(*cell);
                used[i] = true;
            } else {
                let corner_count = cell_corners
                    .iter()
                    .filter(|c| current_corners.contains(c))
                    .count();

                if corner_count > 0 {
                    current_corners.extend(cell_corners);
                    current_cells.push(*cell);
                    used[i] = true;
                }
            }
        }

        if current_cells.len() == initial_count {
            if current_cells.is_empty() {
                break;
            }
            tables.push(std::mem::take(&mut current_cells));
            current_corners.clear();
        }
    }

    if !current_cells.is_empty() {
        tables.push(current_cells);
    }

    tables.sort_by(|a, b| {
        let min_a = a
            .iter()
            .map(|c| (OrderedFloat(c.1), OrderedFloat(c.0)))
            .min()
            .unwrap();
        let min_b = b
            .iter()
            .map(|c| (OrderedFloat(c.1), OrderedFloat(c.0)))
            .min()
            .unwrap();
        min_a.cmp(&min_b)
    });

    tables.into_iter().filter(|t| t.len() > 1).collect()
}
pub(crate) struct TableFinder {
    bottom_origin: bool,
    settings: Rc<TfSettings>,
}

impl TableFinder {
    fn new(settings: Rc<TfSettings>, bottom_origin: bool) -> Self {
        TableFinder {
            bottom_origin: bottom_origin,
            settings: settings.clone(),
        }
    }
    pub(crate) fn get_edges(&self, page: &PdfPage) -> HashMap<Orientation, Vec<Edge>> {
        let settings = self.settings.as_ref();
        if (settings.vertiacl_strategy == StrategyType::Text)
            || (settings.horizontal_strategy == StrategyType::Text)
        {
            panic!("Text strategy not implemented")
        }

        let mut edges_all = make_edges(page, self.bottom_origin);
        let mut v_edges = match settings.vertiacl_strategy {
            StrategyType::LinesStrict => edges_all
                .remove(&EdgeType::VerticalLine)
                .unwrap_or_default(),
            StrategyType::Lines => [
                edges_all
                    .remove(&EdgeType::VerticalLine)
                    .unwrap_or_default(),
                edges_all
                    .remove(&EdgeType::VerticalRect)
                    .unwrap_or_default(),
            ]
            .concat(),
            _ => panic!("Text strategy not implemented"),
        };
        filter_edges_by_min_len(&mut v_edges, settings.edge_min_length_prefilter);
        let mut h_edges = match settings.horizontal_strategy {
            StrategyType::LinesStrict => edges_all
                .remove(&EdgeType::HorizontalLine)
                .unwrap_or_default(),
            StrategyType::Lines => [
                edges_all
                    .remove(&EdgeType::HorizontalLine)
                    .unwrap_or_default(),
                edges_all
                    .remove(&EdgeType::HorizontalRect)
                    .unwrap_or_default(),
            ]
            .concat(),
            _ => panic!("Text strategy not implemented"),
        };
        filter_edges_by_min_len(&mut h_edges, settings.edge_min_length_prefilter);

        let edges_prefiltered = HashMap::from([
            (Orientation::Vertical, v_edges),
            (Orientation::Horizontal, h_edges),
        ]);
        let mut edges_merged = merge_edges(
            edges_prefiltered,
            settings.snap_x_tolerance,
            settings.snap_y_tolerance,
            settings.join_x_tolerance,
            settings.join_y_tolerance,
        );
        if let Some(h_edges) = edges_merged.get_mut(&Orientation::Horizontal) {
            filter_edges_by_min_len(h_edges, settings.edge_min_length);
        }
        if let Some(v_edges) = edges_merged.get_mut(&Orientation::Vertical) {
            filter_edges_by_min_len(v_edges, settings.edge_min_length);
        }
        edges_merged
    }

    // pub fn get_all_cells_bbox(&self, page: &PdfPage) -> Vec<BboxKey> {
    //     let mut edges = self.get_edges(page);
    //     let intersections = edges_to_intersections(
    //         &mut edges,
    //         self.settings.intersection_x_tolerance,
    //         self.settings.intersection_y_tolerance,
    //     );
    //     let cells = intersections_to_cells(intersections);
    //     let cell_groups = cells_to_tables(&cells);
    // }
}
