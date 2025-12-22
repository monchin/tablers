use crate::clusters::cluster_objects;
use crate::objects::*;
use crate::words::Word;
use itertools::Itertools;
use ordered_float::OrderedFloat;
use pdfium_render::prelude::*;
use pyo3::prelude::*;
use std::cmp;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
enum EdgeAttr {
    X1,
    Y1,
}

pub(crate) fn words_to_edges_h(words: &[Word], word_threshold: usize) -> Vec<Edge> {
    let by_top = cluster_objects(words, |w: &Word| w.bbox.1, OrderedFloat(1.0));

    let large_clusters: Vec<_> = by_top
        .into_iter()
        .filter(|cluster| cluster.len() >= word_threshold)
        .collect();

    let rects: Vec<BboxKey> = large_clusters
        .iter()
        .filter_map(|cluster| get_objects_bbox(cluster))
        .collect();

    if rects.is_empty() {
        return Vec::new();
    }

    let min_x1 = rects
        .iter()
        .map(|r| r.0)
        .fold(OrderedFloat(f32::INFINITY), cmp::min);
    let max_x2 = rects
        .iter()
        .map(|r| r.2)
        .fold(OrderedFloat(f32::NEG_INFINITY), cmp::max);

    let mut edges = Vec::with_capacity(rects.len() * 2);

    for r in &rects {
        edges.push(Edge {
            x1: min_x1,
            y1: r.1,
            x2: max_x2,
            y2: r.1,
            orientation: Orientation::Horizontal,
            width: OrderedFloat(1.0f32),
            color: PdfColor::new(0, 0, 0, 255),
        });
        edges.push(Edge {
            x1: min_x1,
            y1: r.3,
            x2: max_x2,
            y2: r.3,
            orientation: Orientation::Horizontal,
            width: OrderedFloat(1.0f32),
            color: PdfColor::new(0, 0, 0, 255),
        });
    }

    edges
}

fn get_bbox_overlap(b1: &BboxKey, b2: &BboxKey) -> bool {
    let (b1_x1, b1_y1, b1_x2, b1_y2) = b1;
    let (b2_x1, b2_y1, b2_x2, b2_y2) = b2;
    let (max_x1, max_y1, min_x2, min_y2) = (
        cmp::max(*b1_x1, *b2_x1),
        cmp::max(*b1_y1, *b2_y1),
        cmp::min(*b1_x2, *b2_x2),
        cmp::min(*b1_y2, *b2_y2),
    );
    max_x1 < min_x2 && max_y1 < min_y2
}

pub fn words_to_edges_v(words: &[Word], word_threshold: usize) -> Vec<Edge> {
    let by_x0 = cluster_objects(words, |w| w.bbox.0, OrderedFloat(1.0));
    let by_x1 = cluster_objects(words, |w| w.bbox.2, OrderedFloat(1.0));
    let by_center = cluster_objects(words, |w| (w.bbox.0 + w.bbox.2) / 2.0, OrderedFloat(1.0));

    let mut clusters: Vec<Vec<Word>> = by_x0;
    clusters.extend(by_x1);
    clusters.extend(by_center);

    clusters.sort_by(|a, b| b.len().cmp(&a.len()));
    let large_clusters: Vec<_> = clusters
        .into_iter()
        .filter(|cluster| cluster.len() >= word_threshold)
        .collect();

    let bboxes: Vec<BboxKey> = large_clusters
        .iter()
        .filter_map(|cluster| get_objects_bbox(cluster))
        .collect();

    let mut condensed_bboxes: Vec<BboxKey> = Vec::new();
    for bbox in bboxes {
        let overlap = condensed_bboxes.iter().any(|c| get_bbox_overlap(&bbox, c));
        if !overlap {
            condensed_bboxes.push(bbox);
        }
    }

    if condensed_bboxes.is_empty() {
        return Vec::new();
    }

    let mut sorted_rects: Vec<BboxKey> = condensed_bboxes
        .into_iter()
        .map(|(x1, y1, x2, y2)| (x1, y1, x2, y2))
        .collect();
    sorted_rects.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // 计算边界值
    let max_x2 = sorted_rects
        .iter()
        .map(|r| r.2)
        .fold(OrderedFloat(f32::NEG_INFINITY), cmp::max);
    let min_top = sorted_rects
        .iter()
        .map(|r| r.1)
        .fold(OrderedFloat(f32::INFINITY), cmp::min);
    let max_bottom = sorted_rects
        .iter()
        .map(|r| r.3)
        .fold(OrderedFloat(f32::NEG_INFINITY), cmp::max);

    let mut edges: Vec<Edge> = sorted_rects
        .iter()
        .map(|b| Edge {
            x1: b.0,
            y1: min_top,
            x2: b.0,
            y2: max_bottom,
            width: OrderedFloat(1.0),
            orientation: Orientation::Vertical,
            color: PdfColor::new(0, 0, 0, 255),
        })
        .collect();

    edges.push(Edge {
        x1: max_x2,
        y1: min_top,
        x2: max_x2,
        y2: max_bottom,
        width: OrderedFloat(1.0),
        orientation: Orientation::Vertical,
        color: PdfColor::new(0, 0, 0, 255),
    });

    edges
}

fn move_edge(edge: Edge, orient: Orientation, value: OrderedFloat<f32>) -> Edge {
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

fn snap_objects(edges: Vec<Edge>, attr: EdgeAttr, tolerance: OrderedFloat<f32>) -> Vec<Edge> {
    let orient = match attr {
        EdgeAttr::X1 => Orientation::Vertical,
        EdgeAttr::Y1 => Orientation::Horizontal,
    };
    let attr_getter = match attr {
        EdgeAttr::X1 => |edge: &Edge| edge.x1,
        EdgeAttr::Y1 => |edge: &Edge| edge.y1,
    };
    let clusters = cluster_objects(&edges, attr_getter, tolerance);
    let mut result = Vec::new();
    for cluster in clusters {
        let avg = cluster
            .iter()
            .map(|edge| attr_getter(edge))
            .sum::<OrderedFloat<f32>>()
            / OrderedFloat(cluster.len() as f32);
        for edge in cluster {
            let move_value = avg - attr_getter(&edge);
            result.push(move_edge(edge, orient, move_value));
        }
    }
    result
}

fn join_edge_group(
    edges: Vec<Edge>,
    orient: Orientation,
    tolerance: OrderedFloat<f32>,
) -> Vec<Edge> {
    if edges.is_empty() {
        return vec![];
    }
    let (get_min_prop, get_max_prop): (
        fn(&Edge) -> OrderedFloat<f32>,
        fn(&Edge) -> OrderedFloat<f32>,
    ) = match orient {
        Orientation::Vertical => (|e| e.y1, |e| e.y2),
        Orientation::Horizontal => (|e| e.x1, |e| e.x2),
    };
    let update_last_edge = match orient {
        Orientation::Vertical => |last_edge: &mut Edge, edge: &Edge| {
            last_edge.y2 = edge.y2;
        },
        Orientation::Horizontal => |last_edge: &mut Edge, edge: &Edge| {
            last_edge.x2 = edge.x2;
        },
    };
    let mut sorted_edges: Vec<Edge> = edges
        .into_iter()
        .sorted_by(|a, b| get_min_prop(a).partial_cmp(&get_min_prop(b)).unwrap())
        .collect();
    let mut result = vec![sorted_edges[0].clone()];
    for edge in sorted_edges.iter_mut().skip(1) {
        let last_edge = result.last_mut().unwrap();
        if get_min_prop(edge) <= get_max_prop(last_edge) + tolerance {
            if get_max_prop(edge) > get_max_prop(last_edge) {
                update_last_edge(last_edge, edge);
            }
        } else {
            result.push(edge.clone());
        }
    }
    result
}

fn merge_one_kind_edges(
    mut edges: Vec<Edge>,
    orient: Orientation,
    snap_tolerance: OrderedFloat<f32>,
    join_tolerance: OrderedFloat<f32>,
) -> Vec<Edge> {
    let get_prop: fn(&Edge) -> OrderedFloat<f32> = match orient {
        Orientation::Vertical => |e| e.x1,
        Orientation::Horizontal => |e| e.y1,
    };
    let attr = match orient {
        Orientation::Vertical => EdgeAttr::X1,
        Orientation::Horizontal => EdgeAttr::Y1,
    };

    if snap_tolerance > OrderedFloat(0.0) {
        edges = snap_objects(edges, attr, snap_tolerance);
    }
    edges.sort_by_key(&get_prop);
    edges
        .chunk_by(|e1, e2| get_prop(e1) == get_prop(e2))
        .map(|slice| slice.to_vec())
        .flat_map(|group| {
            let joined = join_edge_group(group, orient, join_tolerance);
            joined
        })
        .collect()
}

pub(crate) fn merge_edges(
    mut edges: HashMap<Orientation, Vec<Edge>>,
    snap_x_tolerance: OrderedFloat<f32>,
    snap_y_tolerance: OrderedFloat<f32>,
    join_x_tolerance: OrderedFloat<f32>,
    join_y_tolerance: OrderedFloat<f32>,
) -> HashMap<Orientation, Vec<Edge>> {
    HashMap::from([
        (
            Orientation::Vertical,
            merge_one_kind_edges(
                edges.remove(&Orientation::Vertical).unwrap_or_default(),
                Orientation::Vertical,
                snap_x_tolerance,
                join_y_tolerance,
            ),
        ),
        (
            Orientation::Horizontal,
            merge_one_kind_edges(
                edges.remove(&Orientation::Horizontal).unwrap_or_default(),
                Orientation::Horizontal,
                snap_y_tolerance,
                join_x_tolerance,
            ),
        ),
    ])
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct Edge {
    pub orientation: Orientation,
    pub x1: OrderedFloat<f32>,
    pub y1: OrderedFloat<f32>,
    pub x2: OrderedFloat<f32>,
    pub y2: OrderedFloat<f32>,
    pub width: OrderedFloat<f32>, // Stroke width
    pub color: PdfColor,          // Stroke color
}

impl Edge {
    pub(crate) fn to_bbox_key(&self) -> BboxKey {
        (self.x1, self.y1, self.x2, self.y2)
    }
}
#[pymethods]
impl Edge {
    // Getter 手动转换类型
    #[getter]
    fn x1(&self) -> f32 {
        self.x1.into_inner()
    }

    #[getter]
    fn y1(&self) -> f32 {
        self.y1.into_inner()
    }

    #[getter]
    fn x2(&self) -> f32 {
        self.x2.into_inner()
    }

    #[getter]
    fn y2(&self) -> f32 {
        self.y2.into_inner()
    }

    #[getter]
    fn width(&self) -> f32 {
        self.width.into_inner()
    }

    #[getter]
    fn color(&self) -> (u8, u8, u8, u8) {
        (
            self.color.red(),
            self.color.green(),
            self.color.blue(),
            self.color.alpha(),
        )
    }

    #[getter]
    fn orientation(&self) -> &str {
        match self.orientation {
            Orientation::Horizontal => "h",
            Orientation::Vertical => "v",
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Edge(type={}, x1={}, y1={}, x2={}, y2={}, width={}, color=(R{}, G{}, B{}, A{}))",
            self.orientation(),
            self.x1(),
            self.y1(),
            self.x2(),
            self.y2(),
            self.width(),
            self.color.red(),
            self.color.green(),
            self.color.blue(),
            self.color.alpha(),
        )
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.x1 == other.x1 && self.y1 == other.y1 && self.x2 == other.x2 && self.y2 == other.y2
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use ordered_float::OrderedFloat;
//     use pdfium_render::prelude::PdfColor;

//     fn make_test_edge(x1: f32, y1: f32, x2: f32, y2: f32) -> Edge {
//         Edge {
//             edge_type: EdgeType::VerticalLine,
//             x1: OrderedFloat(x1),
//             y1: OrderedFloat(y1),
//             x2: OrderedFloat(x2),
//             y2: OrderedFloat(y2),
//             width: 1.0,
//             color: PdfColor::new(0, 0, 0, 255),
//         }
//     }

//     #[test]
//     fn test_snap_objects() {
//         // 使用 tolerance=1 对齐后，三者的 x1 应该相等
//         let a = make_test_edge(5.0, 20.0, 10.0, 30.0);
//         let b = make_test_edge(6.0, 20.0, 11.0, 30.0);
//         let c = make_test_edge(7.0, 20.0, 12.0, 30.0);

//         let result = snap_objects(vec![a, b, c], EdgeAttr::X1, OrderedFloat(1.0));

//         assert_eq!(result.len(), 3);
//         // 对齐后，三个边的 x1 应该相等（取平均值 (5+6+7)/3 = 6）
//         assert_eq!(result[0].x1, result[1].x1);
//         assert_eq!(result[1].x1, result[2].x1);
//         // 验证平均值
//         assert_eq!(result[0].x1, OrderedFloat(6.0));
//     }

//     #[test]
//     fn test_edge_merging() {
//         use pdfium_render::prelude::Pdfium;

//         let project_root = env!("CARGO_MANIFEST_DIR");

//         #[cfg(target_os = "windows")]
//         let pdfium = Pdfium::new(
//             Pdfium::bind_to_library(&format!("{}/python/tablers/pdfium.dll", project_root))
//                 .unwrap(),
//         );
//         #[cfg(target_os = "macos")]
//         let pdfium = Pdfium::new(
//             Pdfium::bind_to_library(&format!("{}/python/tablers/libpdfium.dylib", project_root))
//                 .unwrap(),
//         );
//         #[cfg(target_os = "linux")]
//         let pdfium = Pdfium::new(
//             Pdfium::bind_to_library(&format!("{}/python/tablers/libpdfium.so", project_root))
//                 .unwrap(),
//         );

//         let pdf_path = format!("{}/tests/data/edge-test.pdf", project_root);
//         let doc = pdfium.load_pdf_from_file(&pdf_path, None).unwrap();
//         let page = doc.pages().get(0).unwrap();
//         let pdf_page = crate::pages::Page::new(unsafe { std::mem::transmute(page) }, 0, false);

//         let edges_by_type = make_edges(&pdf_page, true);

//         // 原始边数 364
//         let total: usize = edges_by_type.values().map(|v| v.len()).sum();
//         assert_eq!(total, 364);

//         // 辅助函数：EdgeType -> Orientation
//         let to_orient = |mut e: HashMap<EdgeType, Vec<Edge>>| -> HashMap<Orientation, Vec<Edge>> {
//             HashMap::from([
//                 (
//                     Orientation::Vertical,
//                     [
//                         e.remove(&EdgeType::VerticalLine).unwrap_or_default(),
//                         e.remove(&EdgeType::VerticalRect).unwrap_or_default(),
//                     ]
//                     .concat(),
//                 ),
//                 (
//                     Orientation::Horizontal,
//                     [
//                         e.remove(&EdgeType::HorizontalLine).unwrap_or_default(),
//                         e.remove(&EdgeType::HorizontalRect).unwrap_or_default(),
//                     ]
//                     .concat(),
//                 ),
//             ])
//         };
//         let count =
//             |e: &HashMap<Orientation, Vec<Edge>>| -> usize { e.values().map(|v| v.len()).sum() };

//         // 测试1: snap_x=3, snap_y=3, join_x=3, join_y=3 => 46
//         let merged = merge_edges(
//             to_orient(edges_by_type.clone()),
//             OrderedFloat(3.0),
//             OrderedFloat(3.0),
//             OrderedFloat(3.0),
//             OrderedFloat(3.0),
//         );
//         assert_eq!(count(&merged), 46);

//         // 测试2: snap_x=3, snap_y=3, join_x=3, join_y=0 => 52
//         let merged = merge_edges(
//             to_orient(edges_by_type.clone()),
//             OrderedFloat(3.0),
//             OrderedFloat(3.0),
//             OrderedFloat(3.0),
//             OrderedFloat(0.0),
//         );
//         assert_eq!(count(&merged), 52);

//         // 测试3: snap_x=0, snap_y=3, join_x=3, join_y=3 => 94
//         let merged = merge_edges(
//             to_orient(edges_by_type.clone()),
//             OrderedFloat(0.0001),
//             OrderedFloat(3.0),
//             OrderedFloat(3.0),
//             OrderedFloat(3.0),
//         );
//         assert_eq!(count(&merged), 56);

//         // 测试4: snap_x=3, snap_y=0, join_x=3, join_y=3 => 174
//         let merged = merge_edges(
//             to_orient(edges_by_type.clone()),
//             OrderedFloat(3.0),
//             OrderedFloat(0.0001),
//             OrderedFloat(3.0),
//             OrderedFloat(3.0),
//         );
//         assert_eq!(count(&merged), 166);
//     }
// }
