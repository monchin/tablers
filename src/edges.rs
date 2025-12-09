use pdfium_render::prelude::*;
use std::collections::HashMap;
use std::cmp;
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
