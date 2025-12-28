use crate::clusters::cluster_objects;
use crate::objects::*;
use crate::settings::*;
use crate::words::Word;
use crate::words::*;
use itertools::Itertools;
use ordered_float::OrderedFloat;
use pdfium_render::prelude::*;
use pyo3::prelude::*;
use std::cmp;
use std::collections::HashMap;
use std::rc::Rc;

/// Attribute type for edge snapping operations.
#[derive(Debug, Clone, Copy)]
enum EdgeAttr {
    /// X1 coordinate (left edge for vertical lines).
    X1,
    /// Y1 coordinate (top edge for horizontal lines).
    Y1,
}

/// Converts words into horizontal edges based on text alignment.
///
/// This function clusters words by their top position and creates horizontal
/// edges from the bounding boxes of sufficiently large clusters.
///
/// # Arguments
///
/// * `words` - A slice of Word objects to analyze.
/// * `word_threshold` - Minimum number of words required in a cluster to create edges.
///
/// # Returns
///
/// A vector of horizontal Edge objects derived from word positions.
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

/// Checks if two bounding boxes overlap.
///
/// # Arguments
///
/// * `b1` - The first bounding box.
/// * `b2` - The second bounding box.
///
/// # Returns
///
/// `true` if the bounding boxes overlap, `false` otherwise.
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

/// Converts words into vertical edges based on text alignment.
///
/// This function clusters words by their left, right, and center x-positions
/// and creates vertical edges from sufficiently large, non-overlapping clusters.
///
/// # Arguments
///
/// * `words` - A slice of Word objects to analyze.
/// * `word_threshold` - Minimum number of words required in a cluster to create edges.
///
/// # Returns
///
/// A vector of vertical Edge objects derived from word positions.
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

/// Moves an edge by a specified value in the given orientation.
///
/// # Arguments
///
/// * `edge` - The edge to move.
/// * `orient` - The orientation determining how to move the edge.
/// * `value` - The amount to move the edge.
///
/// # Returns
///
/// A new Edge with updated coordinates.
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

/// Snaps edges together that are within a specified tolerance.
///
/// Edges with similar positions (within tolerance) are moved to share
/// the same coordinate, which is the average of the cluster.
///
/// # Arguments
///
/// * `edges` - The edges to snap.
/// * `attr` - Which attribute (X1 or Y1) to use for snapping.
/// * `tolerance` - Maximum distance for edges to be considered part of the same group.
///
/// # Returns
///
/// A vector of edges with snapped coordinates.
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

/// Joins overlapping or adjacent edges within a group.
///
/// Edges that overlap or are within the tolerance are merged into single edges.
///
/// # Arguments
///
/// * `edges` - A vector of edges to join.
/// * `orient` - The orientation of the edges.
/// * `tolerance` - Maximum gap between edges to consider them joinable.
///
/// # Returns
///
/// A vector of joined edges.
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

/// Merges edges of a single orientation by snapping and joining.
///
/// First snaps nearby edges together, then joins overlapping edges.
///
/// # Arguments
///
/// * `edges` - The edges to merge.
/// * `orient` - The orientation of all edges.
/// * `snap_tolerance` - Tolerance for snapping edges together.
/// * `join_tolerance` - Tolerance for joining overlapping edges.
///
/// # Returns
///
/// A vector of merged edges.
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

/// Merges both horizontal and vertical edges with specified tolerances.
///
/// # Arguments
///
/// * `edges` - A HashMap of edges grouped by orientation.
/// * `snap_x_tolerance` - X-axis tolerance for snapping vertical edges.
/// * `snap_y_tolerance` - Y-axis tolerance for snapping horizontal edges.
/// * `join_x_tolerance` - X-axis tolerance for joining horizontal edges.
/// * `join_y_tolerance` - Y-axis tolerance for joining vertical edges.
///
/// # Returns
///
/// A HashMap of merged edges grouped by orientation.
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

/// Represents a line edge extracted from a PDF page.
///
/// An edge can be either horizontal or vertical and includes
/// position, width, and color information.
#[pyclass]
#[derive(Debug, Clone)]
pub struct Edge {
    /// The orientation of the edge (horizontal or vertical).
    pub orientation: Orientation,
    /// The left x-coordinate of the edge.
    pub x1: OrderedFloat<f32>,
    /// The top y-coordinate of the edge.
    pub y1: OrderedFloat<f32>,
    /// The right x-coordinate of the edge.
    pub x2: OrderedFloat<f32>,
    /// The bottom y-coordinate of the edge.
    pub y2: OrderedFloat<f32>,
    /// The stroke width of the edge.
    pub width: OrderedFloat<f32>,
    /// The stroke color of the edge.
    pub color: PdfColor,
}

impl Edge {
    /// Converts the edge coordinates to a bounding box key.
    ///
    /// # Returns
    ///
    /// A tuple of (x1, y1, x2, y2) coordinates.
    pub(crate) fn to_bbox_key(&self) -> BboxKey {
        (self.x1, self.y1, self.x2, self.y2)
    }
}
#[pymethods]
impl Edge {
    /// Returns the left x-coordinate of the edge.
    #[getter]
    fn x1(&self) -> f32 {
        self.x1.into_inner()
    }

    /// Returns the top y-coordinate of the edge.
    #[getter]
    fn y1(&self) -> f32 {
        self.y1.into_inner()
    }

    /// Returns the right x-coordinate of the edge.
    #[getter]
    fn x2(&self) -> f32 {
        self.x2.into_inner()
    }

    /// Returns the bottom y-coordinate of the edge.
    #[getter]
    fn y2(&self) -> f32 {
        self.y2.into_inner()
    }

    /// Returns the stroke width of the edge.
    #[getter]
    fn width(&self) -> f32 {
        self.width.into_inner()
    }

    /// Returns the color as an RGBA tuple.
    #[getter]
    fn color(&self) -> (u8, u8, u8, u8) {
        (
            self.color.red(),
            self.color.green(),
            self.color.blue(),
            self.color.alpha(),
        )
    }

    /// Returns the orientation as a string ("h" or "v").
    #[getter]
    fn orientation(&self) -> &str {
        match self.orientation {
            Orientation::Horizontal => "h",
            Orientation::Vertical => "v",
        }
    }

    /// Returns a string representation of the edge for debugging.
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

    /// Checks equality based on coordinates.
    fn __eq__(&self, other: &Self) -> bool {
        self.x1 == other.x1 && self.y1 == other.y1 && self.x2 == other.x2 && self.y2 == other.y2
    }
}

/// Creates edges from PDF page objects based on the specified strategy.
///
/// This function extracts edges from lines, rectangles, and optionally from
/// text alignment based on the provided settings.
///
/// # Arguments
///
/// * `objects` - The extracted PDF objects (lines, rects, chars).
/// * `tf_settings` - Table finder settings controlling the extraction strategy.
///
/// # Returns
///
/// A HashMap of edges grouped by orientation (horizontal/vertical).
pub(crate) fn make_edges(
    objects: &Objects,
    tf_settings: Rc<TfSettings>,
) -> HashMap<Orientation, Vec<Edge>> {
    let (snap_x_tol, snap_y_tol) = (tf_settings.snap_x_tolerance, tf_settings.snap_y_tolerance);
    let lines = &objects.lines;
    let rects = &objects.rects;
    let mut edges = HashMap::new();
    edges.insert(Orientation::Horizontal, Vec::new());
    edges.insert(Orientation::Vertical, Vec::new());

    let (h_strat, v_strat) = (
        tf_settings.horizontal_strategy,
        tf_settings.vertical_strategy,
    );
    if h_strat == StrategyType::Text || v_strat == StrategyType::Text {
        let words = WordExtractor::new(&tf_settings.text_settings).extract_words(&objects.chars);
        if h_strat == StrategyType::Text {
            edges
                .get_mut(&Orientation::Horizontal)
                .unwrap()
                .extend(words_to_edges_h(&words, tf_settings.min_words_horizontal));
        }
        if v_strat == StrategyType::Text {
            edges
                .get_mut(&Orientation::Vertical)
                .unwrap()
                .extend(words_to_edges_v(&words, tf_settings.min_words_vertical));
        }
    }

    if ((h_strat | 0b11u8) != 0) || ((v_strat | 0b11u8) != 0) {
        // 0b11: Lines or LinesStrict
        for line in lines {
            if line.line_type == LineType::Straight {
                let (p1, p2) = (line.points[0], line.points[1]);
                if ((v_strat | 0b11u8) != 0) && ((p1.0 - p2.0).abs() < snap_x_tol.into_inner()) {
                    edges.get_mut(&Orientation::Vertical).unwrap().push(Edge {
                        orientation: Orientation::Vertical,
                        x1: p1.0,
                        y1: cmp::min(p1.1, p2.1),
                        x2: p1.0,
                        y2: cmp::max(p1.1, p2.1),
                        width: line.width,
                        color: line.color,
                    });
                } else if ((h_strat | 0b11u8) != 0)
                    && ((p1.1 - p2.1).abs() < snap_y_tol.into_inner())
                {
                    edges.get_mut(&Orientation::Horizontal).unwrap().push(Edge {
                        orientation: Orientation::Horizontal,
                        x1: cmp::min(p1.0, p2.0),
                        y1: p1.1,
                        x2: cmp::max(p1.0, p2.0),
                        y2: p1.1,
                        width: line.width,
                        color: line.color,
                    })
                }
            }
        }

        for rect in rects {
            if ((v_strat | 0b11u8) != 0) && (rect.bbox.2 - rect.bbox.0 < snap_x_tol) {
                let x = (rect.bbox.0 + rect.bbox.2) / 2.0;
                edges.get_mut(&Orientation::Vertical).unwrap().push(Edge {
                    orientation: Orientation::Vertical,
                    x1: x,
                    y1: rect.bbox.1,
                    x2: x,
                    y2: rect.bbox.3,
                    width: rect.bbox.2 - rect.bbox.0,
                    color: rect.fill_color,
                });
            } else if ((h_strat | 0b11u8) != 0) && (rect.bbox.3 - rect.bbox.1 < snap_y_tol) {
                let y = (rect.bbox.1 + rect.bbox.3) / 2.0;
                edges.get_mut(&Orientation::Horizontal).unwrap().push(Edge {
                    orientation: Orientation::Horizontal,
                    x1: rect.bbox.0,
                    y1: y,
                    x2: rect.bbox.2,
                    y2: y,
                    width: rect.bbox.3 - rect.bbox.1,
                    color: rect.fill_color,
                })
            } else {
                if h_strat == StrategyType::Lines {
                    edges.get_mut(&Orientation::Horizontal).unwrap().push(Edge {
                        orientation: Orientation::Horizontal,
                        x1: rect.bbox.0,
                        y1: rect.bbox.1,
                        x2: rect.bbox.2,
                        y2: rect.bbox.1,
                        width: OrderedFloat::from(rect.stroke_width),
                        color: rect.stroke_color,
                    });
                    edges.get_mut(&Orientation::Horizontal).unwrap().push(Edge {
                        orientation: Orientation::Horizontal,
                        x1: rect.bbox.0,
                        y1: rect.bbox.3,
                        x2: rect.bbox.2,
                        y2: rect.bbox.3,
                        width: OrderedFloat::from(rect.stroke_width),
                        color: rect.stroke_color,
                    });
                }
                if v_strat == StrategyType::Lines {
                    edges.get_mut(&Orientation::Vertical).unwrap().push(Edge {
                        orientation: Orientation::Vertical,
                        x1: rect.bbox.0,
                        y1: rect.bbox.1,
                        x2: rect.bbox.0,
                        y2: rect.bbox.3,
                        width: OrderedFloat::from(rect.stroke_width),
                        color: rect.stroke_color,
                    });
                    edges.get_mut(&Orientation::Vertical).unwrap().push(Edge {
                        orientation: Orientation::Vertical,
                        x1: rect.bbox.2,
                        y1: rect.bbox.1,
                        x2: rect.bbox.2,
                        y2: rect.bbox.3,
                        width: OrderedFloat::from(rect.stroke_width),
                        color: rect.stroke_color,
                    });
                }
            }
        }
    }

    edges
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::load_pdfium;
    use ordered_float::OrderedFloat;
    use pdfium_render::prelude::PdfColor;

    fn make_test_edge(x1: f32, y1: f32, x2: f32, y2: f32) -> Edge {
        Edge {
            orientation: Orientation::Vertical,
            x1: OrderedFloat(x1),
            y1: OrderedFloat(y1),
            x2: OrderedFloat(x2),
            y2: OrderedFloat(y2),
            width: OrderedFloat(1.0),
            color: PdfColor::new(0, 0, 0, 255),
        }
    }

    #[test]
    fn test_snap_objects() {
        let a = make_test_edge(5.0, 20.0, 10.0, 30.0);
        let b = make_test_edge(6.0, 20.0, 11.0, 30.0);
        let c = make_test_edge(7.0, 20.0, 12.0, 30.0);

        let result = snap_objects(vec![a, b, c], EdgeAttr::X1, OrderedFloat(1.0));

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].x1, result[1].x1);
        assert_eq!(result[1].x1, result[2].x1);
        // 验证平均值
        assert_eq!(result[0].x1, OrderedFloat(6.0));
    }

    #[test]
    fn test_edge_merging() {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let pdfium = load_pdfium();

        let pdf_path = format!("{}/tests/data/edge-test.pdf", project_root);
        let doc = pdfium.load_pdf_from_file(&pdf_path, None).unwrap();
        let page = doc.pages().get(0).unwrap();
        let pdf_page = crate::pages::Page::new(unsafe { std::mem::transmute(page) }, 0);

        let edges_by_orientation = make_edges(
            &pdf_page.objects.borrow().as_ref().unwrap(),
            Rc::new(TfSettings::default()),
        );

        let total: usize = edges_by_orientation.values().map(|v| v.len()).sum();
        assert_eq!(total, 202);

        let count =
            |e: &HashMap<Orientation, Vec<Edge>>| -> usize { e.values().map(|v| v.len()).sum() };

        let merged = merge_edges(
            edges_by_orientation.clone(),
            OrderedFloat(3.0),
            OrderedFloat(3.0),
            OrderedFloat(3.0),
            OrderedFloat(3.0),
        );
        assert_eq!(count(&merged), 46);

        let merged = merge_edges(
            edges_by_orientation.clone(),
            OrderedFloat(3.0),
            OrderedFloat(3.0),
            OrderedFloat(3.0),
            OrderedFloat(0.0),
        );
        assert_eq!(count(&merged), 52);

        let merged = merge_edges(
            edges_by_orientation.clone(),
            OrderedFloat(0.0001),
            OrderedFloat(3.0),
            OrderedFloat(3.0),
            OrderedFloat(3.0),
        );
        assert_eq!(count(&merged), 47);

        let merged = merge_edges(
            edges_by_orientation.clone(),
            OrderedFloat(3.0),
            OrderedFloat(0.0001),
            OrderedFloat(3.0),
            OrderedFloat(3.0),
        );
        assert_eq!(count(&merged), 88);
    }
}
