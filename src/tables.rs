use crate::objects::*;
use crate::pages::Page;
use crate::edges::*;
use ordered_float::OrderedFloat;
use pdfium_render::prelude::PdfColor;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::cmp;
static DEFAULT_SNAP_TOLERANCE: f32 = 3.0;
static DEFAULT_JOIN_TOLERANCE: f32 = 3.0;
static DEFAULT_INTERSECTION_TOLERANCE: f32 = 3.0;

#[pyclass]
#[derive(Debug, Clone)]
pub struct TableCell {
    pub text: String,
    pub bbox: BboxKey,
}
#[pymethods]
impl TableCell {
    #[getter]
    fn text(&self) -> &str {
        &self.text
    }

    #[getter]
    fn bbox(&self) -> (f32, f32, f32, f32) {
        (
            self.bbox.0.into_inner(),
            self.bbox.1.into_inner(),
            self.bbox.2.into_inner(),
            self.bbox.3.into_inner(),
        )
    }
}
#[pyclass]
pub struct Table {
    pub cells: Vec<TableCell>,
    pub bbox: BboxKey,
    pub page_index: usize,
    pub text_extracted: bool,
}
#[pymethods]
impl Table {
    #[getter]
    fn cells(&self) -> Vec<TableCell> {
        self.cells.clone()
    }

    #[getter]
    fn bbox(&self) -> (f32, f32, f32, f32) {
        (
            self.bbox.0.into_inner(),
            self.bbox.1.into_inner(),
            self.bbox.2.into_inner(),
            self.bbox.3.into_inner(),
        )
    }

    #[getter]
    fn page_idx(&self) -> usize {
        self.page_index
    }

    #[getter]
    fn text_extracted(&self) -> bool {
        self.text_extracted
    }
}
fn get_table_bbox(cells_bbox: &[BboxKey]) -> BboxKey {
    let x1 = cells_bbox
        .iter()
        .map(|c| OrderedFloat(c.0))
        .min()
        .unwrap()
        .into_inner();

    let y1 = cells_bbox
        .iter()
        .map(|c| OrderedFloat(c.1))
        .min()
        .unwrap()
        .into_inner();

    let x2 = cells_bbox
        .iter()
        .map(|c| OrderedFloat(c.2))
        .max()
        .unwrap()
        .into_inner();

    let y2 = cells_bbox
        .iter()
        .map(|c| OrderedFloat(c.3))
        .max()
        .unwrap()
        .into_inner();

    (x1, y1, x2, y2)
}

impl Table {
    pub fn new(page_idx: usize, cells_bbox: &[BboxKey], extract_text: bool) -> Self {
        let bbox = get_table_bbox(cells_bbox);
        let cells;
        if !extract_text {
            cells = cells_bbox
                .iter()
                .map(|bbox| TableCell {
                    text: "".to_string(),
                    bbox: *bbox,
                })
                .collect();
            Self {
                cells,
                bbox,
                page_index: page_idx,
                text_extracted: false,
            }
        } else {
            panic!("extract_text = true not implemented");
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyType {
    Lines,
    LinesStrict,
    Text,
}

#[derive(Debug, Clone)]
#[pyclass]
pub struct TfSettings {
    pub vertical_strategy: StrategyType,
    pub horizontal_strategy: StrategyType,
    pub snap_x_tolerance: OrderedFloat<f32>,
    pub snap_y_tolerance: OrderedFloat<f32>,
    pub join_x_tolerance: OrderedFloat<f32>,
    pub join_y_tolerance: OrderedFloat<f32>,
    pub edge_min_length: OrderedFloat<f32>,
    pub edge_min_length_prefilter: OrderedFloat<f32>,
    pub intersection_x_tolerance: OrderedFloat<f32>,
    pub intersection_y_tolerance: OrderedFloat<f32>,
}
impl Default for TfSettings {
    fn default() -> Self {
        TfSettings {
            vertical_strategy: StrategyType::Lines,
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

#[pymethods]
impl TfSettings {
    #[new]
    #[pyo3(signature = (**kwargs))]
    pub fn py_new(kwargs: Option<&Bound<'_, PyDict>>) -> Self {
        let strategy_str_to_enum = |strategy_str: &str| -> StrategyType {
            match strategy_str {
                "lines" => StrategyType::Lines,
                "lines_strict" => StrategyType::LinesStrict,
                "text" => StrategyType::Text,
                _ => panic!("Invalid strategy: {}", strategy_str),
            }
        };
        let mut settings = TfSettings::default();

        if let Some(kwargs) = kwargs {
            for (key, value) in kwargs.iter() {
                let key = key.to_string();
                match key.as_str() {
                    "vertical_strategy" => {
                        settings.vertical_strategy = strategy_str_to_enum(value.extract().unwrap())
                    }
                    "horizontal_strategy" => {
                        settings.horizontal_strategy =
                            strategy_str_to_enum(value.extract().unwrap())
                    }
                    "snap_x_tolerance" => {
                        settings.snap_x_tolerance =
                            OrderedFloat::from(value.extract::<f32>().unwrap())
                    }
                    "snap_y_tolerance" => {
                        settings.snap_y_tolerance =
                            OrderedFloat::from(value.extract::<f32>().unwrap())
                    }
                    "join_x_tolerance" => {
                        settings.join_x_tolerance =
                            OrderedFloat::from(value.extract::<f32>().unwrap())
                    }
                    "join_y_tolerance" => {
                        settings.join_y_tolerance =
                            OrderedFloat::from(value.extract::<f32>().unwrap())
                    }
                    "edge_min_length" => {
                        settings.edge_min_length =
                            OrderedFloat::from(value.extract::<f32>().unwrap())
                    }
                    "edge_min_length_prefilter" => {
                        settings.edge_min_length_prefilter =
                            OrderedFloat::from(value.extract::<f32>().unwrap())
                    }
                    "intersection_x_tolerance" => {
                        settings.intersection_x_tolerance =
                            OrderedFloat::from(value.extract::<f32>().unwrap())
                    }
                    "intersection_y_tolerance" => {
                        settings.intersection_y_tolerance =
                            OrderedFloat::from(value.extract::<f32>().unwrap())
                    }
                    _ => (), // Ignore unknown settings
                }
            }
        }
        settings
    }
}

fn filter_edges_by_min_len(edges: &mut Vec<Edge>, min_len: OrderedFloat<f32>) {
    edges.retain(|edge| match edge.orientation {
        Orientation::Horizontal => (edge.x2 - edge.x1) >= min_len,
        Orientation::Vertical => (edge.y2 - edge.y1) >= min_len,
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
    pub(crate) fn get_edges(&self, page: &Page) -> HashMap<Orientation, Vec<Edge>> {
        let settings = self.settings.as_ref();
        if (settings.vertical_strategy == StrategyType::Text)
            || (settings.horizontal_strategy == StrategyType::Text)
        {
            panic!("Text strategy not implemented")
        }

        let objects_opt = page.objects.borrow();
        let objects = objects_opt.as_ref().expect("Objects should be extracted");
        let mut edges_all = make_edges(objects, self.settings.clone());
        let mut v_edges = edges_all
                    .remove(&Orientation::Vertical)
                    .unwrap_or_default();
        filter_edges_by_min_len(&mut v_edges, settings.edge_min_length_prefilter);
         let mut h_edges = edges_all
                    .remove(&Orientation::Horizontal)
                    .unwrap_or_default();
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
}

pub fn find_tables(
    pdf_page: &Page,
    tf_settings: Rc<TfSettings>,
    bottom_origin: bool,
    extract_text: bool,
) -> (Vec<BboxKey>, Vec<Table>) {
    let table_finder = TableFinder::new(tf_settings.clone(), bottom_origin);
    let edges = table_finder.get_edges(pdf_page);
    let intersections = edges_to_intersections(
        &mut edges.clone(),
        table_finder.settings.intersection_x_tolerance,
        table_finder.settings.intersection_y_tolerance,
    );
    let cells = intersections_to_cells(intersections);
    let tables_bbox = cells_to_tables(&cells);
    let tables = tables_bbox
        .iter()
        .map(|table_cells_bbox| Table::new(pdf_page.page_idx, table_cells_bbox, extract_text))
        .collect();
    return (cells, tables);
}




fn make_edges(objects: &Objects, tf_settings: Rc<TfSettings>) -> HashMap<Orientation, Vec<Edge>> {
    let (snap_x_tol, snap_y_tol) = (tf_settings.snap_x_tolerance, tf_settings.snap_y_tolerance);
    let lines = &objects.lines;
    let rects = &objects.rects;
    let mut edges = HashMap::new();
    edges.insert(Orientation::Horizontal, Vec::new());
    edges.insert(Orientation::Vertical, Vec::new());

    let (h_strat, v_strat) = (tf_settings.horizontal_strategy, tf_settings.vertical_strategy); 
    if h_strat == StrategyType::Text || v_strat == StrategyType::Text {
        panic!("Text strategy not implemented yet");
    }

    for line in lines {
        if line.line_type == LineType::Straight {
            let (p1, p2) = (line.points[0], line.points[1]);
            if (p1.0 - p2.0).abs() < snap_x_tol.into_inner() {
                edges.get_mut(&Orientation::Vertical).unwrap().push(Edge {
                    orientation: Orientation::Vertical,
                    x1: p1.0,
                    y1: cmp::min(p1.1, p2.1),
                    x2: p1.0,
                    y2: cmp::max(p1.1, p2.1),
                    width: line.width,
                    color: line.color,
                });
            } else if (p1.1 - p2.1).abs() < snap_y_tol.into_inner() {
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
        if rect.bbox.2 - rect.bbox.0 < snap_x_tol {
            let x = (rect.bbox.0 + rect.bbox.2) / 2.0;
            edges.get_mut(&Orientation::Vertical).unwrap().push(Edge {
                orientation: Orientation::Vertical,
                x1: x,
                y1: rect.bbox.1,
                x2: x,
                y2: rect.bbox.3,
                width: (rect.bbox.2 - rect.bbox.0).into_inner(),
                color: rect.fill_color,
            });
        } else if rect.bbox.3 - rect.bbox.1 < snap_y_tol {
            let y = (rect.bbox.1 + rect.bbox.3) / 2.0;
            edges.get_mut(&Orientation::Horizontal).unwrap().push(Edge {
                orientation: Orientation::Horizontal,
                x1: rect.bbox.0,
                y1: y,
                x2: rect.bbox.2,
                y2: y,
                width: (rect.bbox.3 - rect.bbox.1).into_inner(),
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
                    width: rect.stroke_width,
                    color: rect.stroke_color,
                }
                );
                edges.get_mut(&Orientation::Horizontal).unwrap().push(Edge {
                    orientation: Orientation::Horizontal,
                    x1: rect.bbox.0,
                    y1: rect.bbox.3,
                    x2: rect.bbox.2,
                    y2: rect.bbox.3,
                    width: rect.stroke_width,
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
                    width: rect.stroke_width,
                    color: rect.stroke_color,
                });
                edges.get_mut(&Orientation::Vertical).unwrap().push(Edge {
                    orientation: Orientation::Vertical,
                    x1: rect.bbox.2,
                    y1: rect.bbox.1,
                    x2: rect.bbox.2,
                    y2: rect.bbox.3,
                    width: rect.stroke_width,
                    color: rect.stroke_color
                });
            }
        }
    }

    edges
}
