use ordered_float::OrderedFloat;
use pdfium_render::prelude::*;
use std::cmp;
use std::collections::HashMap;

use crate::clusters::cluster_objects;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum EdgeType {
    VerticalLine,
    HorizontalLine,
    VerticalRect,
    HorizontalRect,
    // No need to implement curves as we'll not use them
}

impl EdgeType {
    pub(crate) fn all() -> Vec<EdgeType> {
        vec![
            EdgeType::VerticalLine,
            EdgeType::HorizontalLine,
            EdgeType::VerticalRect,
            EdgeType::HorizontalRect,
        ]
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Edge {
    edge_type: EdgeType,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    width: f32,      // Stroke width
    color: PdfColor, // Stroke color
}

#[inline]
fn get_y_with_bottom_origin(y: f32, bottom_origin: bool, page_height: f32) -> f32 {
    match bottom_origin {
        true => y,
        false => page_height - y,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ObjShape {
    Line,
    Rect,
    NoNeed,
}
fn get_obj_shape(obj: &PdfPagePathObject) -> ObjShape {
    let (mut x1, mut y1, mut x2, mut y2) = (0f32, 0f32, 0f32, 0f32);
    let mut edges = Vec::new();
    for seg in obj.segments().iter() {
        match seg.segment_type() {
            PdfPathSegmentType::MoveTo => {
                // First point of the object
                x1 = seg.x().value;
                y1 = seg.y().value;
            }
            PdfPathSegmentType::LineTo => {
                // Second point of the object
                x2 = seg.x().value;
                y2 = seg.y().value;
                if x1 != x2 && y1 != y2 {
                    return ObjShape::NoNeed;
                }
                edges.push((x1, y1, x2, y2));
                x1 = x2;
                y1 = y2;
            }
            _ => {
                return ObjShape::NoNeed;
            }
        }
    }
    match edges.len() {
        1 => ObjShape::Line,
        4 => ObjShape::Rect,
        _ => ObjShape::NoNeed,
    }
}

#[inline]
fn get_edge_type(x1: f32, y1: f32, x2: f32, y2: f32, obj_shape: ObjShape) -> EdgeType {
    if x1 == x2 {
        if obj_shape == ObjShape::Line {
            EdgeType::VerticalLine
        } else {
            EdgeType::VerticalRect
        }
    } else if y1 == y2 {
        if obj_shape == ObjShape::Line {
            EdgeType::HorizontalLine
        } else {
            EdgeType::HorizontalRect
        }
    } else {
        panic!();
    }
}
fn obj2edge(
    obj: &PdfPagePathObject,
    bottom_origin: bool,
    page_height: f32,
    edges: &mut HashMap<EdgeType, Vec<Edge>>,
) {
    if obj.is_stroked().unwrap() == false {
        return; // We don't need non-stroked objects
    }
    let obj_shape = get_obj_shape(obj);
    if obj_shape == ObjShape::NoNeed {
        return;
    }
    let (mut x1, mut y1, mut x2, mut y2) = (0f32, 0f32, 0f32, 0f32);
    let (line_width, line_color) = (
        obj.stroke_width().unwrap().value,
        obj.stroke_color().unwrap(),
    );
    for seg in obj.segments().transform(obj.matrix().unwrap()).iter() {
        match seg.segment_type() {
            PdfPathSegmentType::MoveTo => {
                // First point of the object
                x1 = seg.x().value;
                y1 = get_y_with_bottom_origin(seg.y().value, bottom_origin, page_height);
            }
            PdfPathSegmentType::LineTo => {
                x2 = seg.x().value;
                y2 = get_y_with_bottom_origin(seg.y().value, bottom_origin, page_height);
                let edge_type = get_edge_type(x1, y1, x2, y2, obj_shape);
                edges.entry(edge_type).or_default().push(Edge {
                    edge_type,
                    x1: cmp::min(x1, x2),
                    y1: cmp::min(y1, y2),
                    x2: cmp::max(x1, x2),
                    y2: cmp::max(y1, y2),
                    width: line_width,
                    color: line_color,
                });
                x1 = x2;
                y1 = y2;
            }
            _ => {} // Impossible after filter ObjShape::NoNeed
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub enum Orientation {
    Vertical,
    Horizontal,
}
#[derive(Debug, Clone, Copy)]
enum EdgeAttr {
    X1,
    Y1,
    X2,
    Y2,
}

fn move_edge(edge: Edge, orient: Orientation, value: f32) -> Edge {
    match orient {
        Orientation::Vertical => Edge {
            x1: edge.x1 + value,
            x2: edge.x2 + value,
            ..edge
        },
        Orientation::Horizontal => Edge {
            y1: edge.y1 + value,
            y2: edge.y2 + value,
            ..edge
        },
    }
}

fn snap_objects(edges: Vec<Edge>, attr: EdgeAttr, tolerance: f32) -> Vec<Edge> {
    let orient = match attr {
        EdgeAttr::X1 => Orientation::Vertical,
        EdgeAttr::Y1 => Orientation::Horizontal,
        EdgeAttr::X2 => Orientation::Vertical,
        EdgeAttr::Y2 => Orientation::Horizontal,
    };
    let attr_getter = match attr {
        EdgeAttr::X1 => |edge: &Edge| edge.x1,
        EdgeAttr::Y1 => |edge: &Edge| edge.y1,
        EdgeAttr::X2 => |edge: &Edge| edge.x2,
        EdgeAttr::Y2 => |edge: &Edge| edge.y2,
    };
    let clusters = cluster_objects(edges, attr_getter, tolerance, false);
    let mut result = Vec::new();
    for cluster in clusters {
        avg = cluster.iter().map(|edge| attr_getter(edge)).sum::<f32>() / cluster.len() as f32;
        for edge in cluster {
            result.push(move_edge(edge, orient, avg - attr_getter(edge)));
        }
    }
    result
}

fn snap_edges(
    edges: HashMap<Orientation, Vec<Edge>>,
    x_tolerance: f32,
    y_tolerance: f32,
) -> HashMap<Orientation, Vec<Edge>> {
    snapped_v = snap_objects(edges[Orientation::Vertical], EdgeAttr::X1, x_tolerance);
    snapped_h = snap_objects(edges[Orientation::Horizontal], EdgeAttr::Y1, y_tolerance);
    HashMap::from([
        (Orientation::Vertical, snapped_v),
        (Orientation::Horizontal, snapped_h),
    ])
}

fn merge_edges(edges: HashMap<Orientation, Vec<Edge>>) -> HashMap<Orientation, Vec<Edge>> {
    let mut result = HashMap::new();
    for (edge_type, edges) in edges {
        let mut edges = edges;
        edges.sort_by(|a, b| a.x1.partial_cmp(&b.x1).unwrap());
        let mut i = 0;
        while i < edges.len() - 1 {}
    }
}

fn join_edge_group(edges: Vec<Edge>, orient: Orientation, tolerance: f32) -> Vec<Edge> {
    if edges.is_empty() {
        vec![]
    }
    let (min_prop, max_prop) = match orient {
        Orientation::Vertical => (EdgeAttr::X1, EdgeAttr::X2),
        Orientation::Horizontal => (EdgeAttr::Y1, EdgeAttr::Y2),
    };
    let (get_min_prop, get_max_prop): (fn(&Edge) -> f32, fn(&Edge) -> f32) = match orient {
        Orientation::Vertical => (|e| e.x1, |e| e.x2),
        Orientation::Horizontal => (|e| e.y1, |e| e.y2),
    };
    let update_last_edge = match orient {
        Orientation::Vertical => |last_edge: &mut Edge, edge: &Edge| {
            last_edge.x2 = edge.x2;
        },
        Orientation::Horizontal => |last_edge: &mut Edge, edge: &Edge| {
            last_edge.y2 = edge.y2;
        },
    };
    let sorted_edges: Vec<Edge> = edges
        .into_iter()
        .sorted_by(|a, b| get_min_prop(a).partial_cmp(&get_min_prop(b)).unwrap())
        .collect();
    let mut result = vec![sorted_edges[0].clone()];
    for edge in sorted_edges.iter_mut().skip(1) {
        let last_edge = result.last_mut().unwrap();
        if (get_min_prop(edge) <= get_max_prop(last_edge) + tolerance)
            && get_max_prop(edge) > get_max_prop(last_edge)
        {
            update_last_edge(last_edge, edge);
        } else {
            result.push(edge.clone());
        }
    }
    result
}

fn merge_one_kind_edges(
    mut edges: Vec<Edge>,
    orient: Orientation,
    snap_tolerance: f32,
    join_tolerance: f32,
) -> Vec<Edge> {
    let get_prop: (fn(&Edge) -> OrderedFloat<f32>) = match orient {
        Orientation::Vertical => |e| OrderedFloat(e.x1),
        Orientation::Horizontal => |e| OrderedFloat(e.y1),
    };
    let attr = match orient {
        Orientation::Vertical => EdgeAttr::X1,
        Orientation::Horizontal => EdgeAttr::Y1,
    };

    if snap_tolerance > 0.0 {
        edges = snap_objects(edges, attr, snap_tolerance);
    }
    edges.sort_by_key(&get_prop);
    use itertools::Itertools;
    edges
        .into_iter()
        .group_by(|edge| get_prop(edge))
        .into_iter()
        .map(|(_, group)| join_edge_group(group.collect(), orient, join_tolerance))
        .flatten()
        .collect()
}

pub(crate) fn merge_edges(
    edges: HashMap<Orientation, Vec<Edge>>,
    snap_x_tolerance: f32,
    snap_y_tolerance: f32,
    join_x_tolerance: f32,
    join_y_tolerance: f32,
) -> HashMap<Orientation, Vec<Edge>> {
    HashMap::from([
        (
            Orientation::Vertical,
            merge_one_kind_edges(
                edges[Orientation::Vertical],
                Orientation::Vertical,
                snap_x_tolerance,
                join_x_tolerance,
            ),
        ),
        (
            Orientation::Horizontal,
            merge_one_kind_edges(
                edges[Orientation::Horizontal],
                Orientation::Horizontal,
                snap_y_tolerance,
                join_y_tolerance,
            ),
        ),
    ])
}

pub(crate) fn make_edges(page: &PdfPage, bottom_origin: bool) -> HashMap<EdgeType, Vec<Edge>> {
    let page_height = page.height().value;
    let mut edges = HashMap::new();
    for each_type in EdgeType::all() {
        edges.insert(each_type, Vec::new());
    }
    for obj in page.objects().iter() {
        if let Some(obj) = obj.as_path_object() {
            obj2edge(obj, bottom_origin, page_height, &mut edges);
        }
    }
    edges
}
