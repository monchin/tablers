use crate::edges::*;
use crate::objects::*;
use crate::pages::Page;
use crate::settings::*;
use crate::words::*;
use ordered_float::OrderedFloat;
use pyo3::prelude::*;
use std::cmp;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::rc::Rc;

/// Specifies whether a cell group represents a row or column.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CellGroupKind {
    /// A horizontal row of cells.
    Row,
    /// A vertical column of cells.
    Column,
}

/// A group of table cells arranged in a row or column.
///
/// Cells may be `None` for empty positions in the grid.
pub struct CellGroup<'tab> {
    /// The cells in this group, with `None` for empty positions.
    pub cells: Vec<Option<&'tab TableCell>>,
    /// The bounding box of the entire group.
    pub bbox: BboxKey,
}

impl<'tab> CellGroup<'tab> {
    /// Creates a new CellGroup from a vector of optional cell references.
    ///
    /// # Arguments
    ///
    /// * `cells` - Vector of optional cell references.
    ///
    /// # Returns
    ///
    /// A new CellGroup with computed bounding box.
    pub fn new(cells: Vec<Option<&'tab TableCell>>) -> Self {
        let non_null_cells: Vec<&&TableCell> = cells.iter().filter_map(|c| c.as_ref()).collect();
        let bbox: BboxKey = (
            non_null_cells
                .iter()
                .map(|c| c.bbox.0)
                .fold(OrderedFloat::from(f32::INFINITY), cmp::min),
            non_null_cells
                .iter()
                .map(|c| c.bbox.1)
                .fold(OrderedFloat::from(f32::INFINITY), cmp::min),
            non_null_cells
                .iter()
                .map(|c| c.bbox.2)
                .fold(OrderedFloat::from(f32::NEG_INFINITY), cmp::max),
            non_null_cells
                .iter()
                .map(|c| c.bbox.3)
                .fold(OrderedFloat::from(f32::NEG_INFINITY), cmp::max),
        );
        Self { cells, bbox }
    }
}

/// Escapes a string field for CSV format.
///
/// Fields containing commas, double quotes, or newlines are wrapped in double quotes.
/// Any double quotes within the field are escaped by doubling them.
///
/// # Arguments
///
/// * `field` - The string field to escape.
///
/// # Returns
///
/// The escaped CSV field.
fn escape_csv_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') || field.contains('\r') {
        let escaped = field.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    } else {
        field.to_string()
    }
}

/// Escapes a string field for Markdown table format.
///
/// Pipe characters are escaped with backslash, and newlines are replaced with `<br>`.
///
/// # Arguments
///
/// * `field` - The string field to escape.
///
/// # Returns
///
/// The escaped Markdown field.
fn escape_markdown_field(field: &str) -> String {
    field
        .replace('|', "\\|")
        .replace('\r', "")
        .replace('\n', "<br>")
}

/// Escapes a string field for HTML format.
///
/// Special HTML characters are escaped to their entity equivalents:
/// - `&` becomes `&amp;`
/// - `<` becomes `&lt;`
/// - `>` becomes `&gt;`
/// - `"` becomes `&quot;`
/// - Newlines are replaced with `<br>`
///
/// # Arguments
///
/// * `field` - The string field to escape.
///
/// # Returns
///
/// The escaped HTML field.
fn escape_html_field(field: &str) -> String {
    field
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\r', "")
        .replace('\n', "<br>")
}

/// Gets a coordinate value from a bounding box by axis index.
///
/// # Arguments
///
/// * `cell` - The bounding box.
/// * `axis` - The axis index (0=x1, 1=y1, 2=x2, 3=y2).
///
/// # Returns
///
/// The coordinate value at the specified axis.
///
/// # Panics
///
/// Panics if axis is not in range 0-3.
fn get_axis_value(cell: &BboxKey, axis: usize) -> OrderedFloat<f32> {
    match axis {
        0 => cell.0, // x1
        1 => cell.1, // y1
        2 => cell.2, // x2
        3 => cell.3, // y2
        _ => panic!("Invalid axis"),
    }
}

/// Represents a single cell in a table.
///
/// Each cell has a bounding box and optional text content.
#[pyclass]
#[derive(Debug, Clone)]
pub struct TableCell {
    /// The text content of the cell.
    pub text: String,
    /// The bounding box of the cell.
    pub bbox: BboxKey,
}

#[pymethods]
impl TableCell {
    /// Returns the text content of the cell.
    #[getter]
    fn text(&self) -> &str {
        &self.text
    }

    /// Returns the bounding box as a tuple (x1, y1, x2, y2).
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

/// Represents a table extracted from a PDF page.
///
/// A table consists of cells organized in a grid structure.
#[pyclass]
pub struct Table {
    /// All cells in the table.
    pub cells: Vec<TableCell>,
    /// The bounding box of the entire table.
    pub bbox: BboxKey,
    /// The index of the page containing this table.
    #[pyo3(get)]
    pub page_index: usize,
    /// Whether text has been extracted for cells.
    #[pyo3(get)]
    pub text_extracted: bool,
}
#[pymethods]
impl Table {
    /// Returns a clone of all cells in the table.
    #[getter]
    fn cells(&self) -> Vec<TableCell> {
        self.cells.clone()
    }

    /// Returns the bounding box as a tuple (x1, y1, x2, y2).
    #[getter]
    fn bbox(&self) -> (f32, f32, f32, f32) {
        (
            self.bbox.0.into_inner(),
            self.bbox.1.into_inner(),
            self.bbox.2.into_inner(),
            self.bbox.3.into_inner(),
        )
    }

    /// Converts the table to a CSV formatted string.
    ///
    /// # Returns
    ///
    /// A Result containing the CSV string, or an error if text has not been extracted.
    ///
    /// # Errors
    ///
    /// Returns a PyValueError if text_extracted is false.
    #[pyo3(name = "to_csv")]
    fn py_to_csv(&self) -> PyResult<String> {
        self.to_csv()
            .map_err(pyo3::exceptions::PyValueError::new_err)
    }

    /// Converts the table to a Markdown formatted string.
    ///
    /// # Returns
    ///
    /// A Result containing the Markdown string, or an error if text has not been extracted.
    ///
    /// # Errors
    ///
    /// Returns a PyValueError if text_extracted is false.
    #[pyo3(name = "to_markdown")]
    fn py_to_markdown(&self) -> PyResult<String> {
        self.to_markdown()
            .map_err(pyo3::exceptions::PyValueError::new_err)
    }

    /// Converts the table to an HTML formatted string.
    ///
    /// # Returns
    ///
    /// A Result containing the HTML string, or an error if text has not been extracted.
    ///
    /// # Errors
    ///
    /// Returns a PyValueError if text_extracted is false.
    #[pyo3(name = "to_html")]
    fn py_to_html(&self) -> PyResult<String> {
        self.to_html()
            .map_err(pyo3::exceptions::PyValueError::new_err)
    }
}

/// Computes the bounding box of a table from its cell bounding boxes.
///
/// # Arguments
///
/// * `cells_bbox` - A slice of cell bounding boxes.
///
/// # Returns
///
/// The combined bounding box encompassing all cells.
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
    /// Creates a new Table from cell bounding boxes.
    ///
    /// # Arguments
    ///
    /// * `page_idx` - The page index where the table is located.
    /// * `cells_bbox` - Bounding boxes for all cells.
    /// * `extract_text` - Whether to extract text content.
    /// * `chars` - Optional character array for text extraction.
    /// * `we_settings` - Optional word extraction settings.
    /// * `need_strip` - Whether to strip leading/trailing whitespace from cell text.
    ///
    /// # Returns
    ///
    /// A new Table instance.
    pub fn new(
        page_idx: usize,
        cells_bbox: &[BboxKey],
        extract_text: bool,
        chars: Option<&[Char]>,
        we_settings: Option<&WordsExtractSettings>,
        need_strip: bool,
    ) -> Self {
        let bbox = get_table_bbox(cells_bbox);
        let cells = cells_bbox
            .iter()
            .map(|bbox| TableCell {
                text: "".to_string(),
                bbox: *bbox,
            })
            .collect();
        let mut slf = Self {
            cells,
            bbox,
            page_index: page_idx,
            text_extracted: false,
        };
        if extract_text {
            match chars {
                Some(chars) => slf.extract_text(chars, we_settings, need_strip),
                None => panic!("No chars provided"),
            };
        };
        slf
    }

    /// Gets all rows or columns from the table cells.
    ///
    /// # Arguments
    ///
    /// * `cells` - The table cells.
    /// * `kind` - Whether to get rows or columns.
    ///
    /// # Returns
    ///
    /// A vector of CellGroup representing rows or columns.
    fn get_rows_or_cols<'tab>(
        cells: &'tab [TableCell],
        kind: CellGroupKind,
    ) -> Vec<CellGroup<'tab>> {
        let axis: usize = if kind == CellGroupKind::Row { 0 } else { 1 };
        let antiaxis: usize = if axis == 0 { 1 } else { 0 };

        let mut indices: Vec<usize> = (0..cells.len()).collect();
        indices.sort_by(|&a, &b| {
            let cell_a = &cells[a];
            let cell_b = &cells[b];
            let a_anti = get_axis_value(&cell_a.bbox, antiaxis);
            let b_anti = get_axis_value(&cell_b.bbox, antiaxis);
            let a_axis = get_axis_value(&cell_a.bbox, axis);
            let b_axis = get_axis_value(&cell_b.bbox, axis);

            a_anti
                .partial_cmp(&b_anti)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(
                    a_axis
                        .partial_cmp(&b_axis)
                        .unwrap_or(std::cmp::Ordering::Equal),
                )
        });

        let sorted_refs: Vec<&'tab TableCell> = indices.iter().map(|&i| &cells[i]).collect();

        let xs: BTreeSet<OrderedFloat<f32>> = cells
            .iter()
            .map(|cell| get_axis_value(&cell.bbox, axis))
            .collect();
        let xs: Vec<OrderedFloat<f32>> = xs.into_iter().collect();

        let mut grouped: HashMap<OrderedFloat<f32>, Vec<&TableCell>> = HashMap::new();
        for cell in &sorted_refs {
            let key = get_axis_value(&cell.bbox, antiaxis);
            grouped.entry(key).or_default().push(cell);
        }

        let mut group_keys: Vec<OrderedFloat<f32>> = grouped.keys().copied().collect();
        group_keys.sort();

        let mut rows: Vec<CellGroup> = Vec::new();

        for group in sorted_refs.chunk_by(|a, b| {
            (get_axis_value(&a.bbox, antiaxis) - get_axis_value(&b.bbox, antiaxis)).abs() < 0.001
        }) {
            let xdict: HashMap<OrderedFloat<f32>, &'tab TableCell> = group
                .iter()
                .map(|cell| (get_axis_value(&cell.bbox, axis), *cell))
                .collect();

            let row_data: Vec<Option<&'tab TableCell>> =
                xs.iter().map(|x| xdict.get(x).copied()).collect();

            rows.push(CellGroup::new(row_data));
        }

        rows
    }

    /// Returns all rows in the table.
    pub fn rows(&self) -> Vec<CellGroup<'_>> {
        Self::get_rows_or_cols(&self.cells, CellGroupKind::Row)
    }

    /// Returns all columns in the table.
    pub fn columns(&self) -> Vec<CellGroup<'_>> {
        Self::get_rows_or_cols(&self.cells, CellGroupKind::Column)
    }

    /// Checks if a character's center is within a bounding box.
    ///
    /// # Arguments
    ///
    /// * `char` - The character to check.
    /// * `bbox` - The bounding box to check against.
    ///
    /// # Returns
    ///
    /// `true` if the character center is inside the bounding box.
    #[inline]
    fn char_in_bbox(char: &Char, bbox: &BboxKey) -> bool {
        let v_mid = (char.bbox.1 + char.bbox.3) / 2.0;
        let h_mid = (char.bbox.0 + char.bbox.2) / 2.0;
        let (x1, y1, x2, y2) = *bbox;
        h_mid >= x1 && h_mid < x2 && v_mid >= y1 && v_mid < y2
    }

    /// Extracts text content for all cells in the table.
    ///
    /// # Arguments
    ///
    /// * `chars` - The characters from the page.
    /// * `settings` - Optional word extraction settings.
    /// * `need_strip` - Whether to strip leading/trailing whitespace from cell text.
    pub fn extract_text(
        &mut self,
        chars: &[Char],
        settings: Option<&WordsExtractSettings>,
        need_strip: bool,
    ) {
        let default_settings = WordsExtractSettings::default();
        let base_settings = settings.unwrap_or(&default_settings);
        let word_settings = WordsExtractSettings {
            keep_blank_chars: true, // keep_blank_chars should be true anyway
            ..base_settings.clone()
        };
        let word_extractor = WordExtractor::new(&word_settings);

        for cell in &mut self.cells {
            let cell_chars: Vec<Char> = chars
                .iter()
                .filter(|char| Self::char_in_bbox(char, &cell.bbox))
                .cloned()
                .collect();

            if !cell_chars.is_empty() {
                let words = word_extractor.extract_words(&cell_chars);
                let mut text = words
                    .iter()
                    .map(|w| w.text.replace("\r\n", "\n").replace('\r', "\n"))
                    .collect::<Vec<_>>()
                    .join("");
                if need_strip {
                    text = text.trim().to_string();
                }
                cell.text = text;
            }
        }
        self.text_extracted = true;
    }

    /// Converts the table to a CSV formatted string.
    ///
    /// # Returns
    ///
    /// A Result containing the CSV string, or an error if text has not been extracted.
    ///
    /// # Errors
    ///
    /// Returns an error if `text_extracted` is false.
    pub fn to_csv(&self) -> Result<String, &'static str> {
        if !self.text_extracted {
            return Err("Text has not been extracted. Call extract_text first.");
        }

        let rows = self.rows();
        let csv_rows: Vec<String> = rows
            .iter()
            .map(|row| {
                row.cells
                    .iter()
                    .map(|cell| {
                        let text = cell.map(|c| c.text.as_str()).unwrap_or("");
                        escape_csv_field(text)
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .collect();

        Ok(csv_rows.join("\n"))
    }

    /// Converts the table to a Markdown formatted string.
    ///
    /// # Returns
    ///
    /// A Result containing the Markdown table string, or an error if text has not been extracted.
    ///
    /// # Errors
    ///
    /// Returns an error if `text_extracted` is false.
    pub fn to_markdown(&self) -> Result<String, &'static str> {
        if !self.text_extracted {
            return Err("Text has not been extracted. Call extract_text first.");
        }

        let rows = self.rows();
        if rows.is_empty() {
            return Ok(String::new());
        }

        let col_count = rows.first().map(|r| r.cells.len()).unwrap_or(0);
        if col_count == 0 {
            return Ok(String::new());
        }

        let mut md_rows: Vec<String> = Vec::new();

        // Generate all rows
        for row in &rows {
            let row_str = row
                .cells
                .iter()
                .map(|cell| {
                    let text = cell.map(|c| c.text.as_str()).unwrap_or("");
                    escape_markdown_field(text)
                })
                .collect::<Vec<_>>()
                .join(" | ");
            md_rows.push(format!("| {} |", row_str));
        }

        // Insert separator after header row
        let separator = format!("| {} |", vec!["---"; col_count].join(" | "));
        if md_rows.len() > 1 {
            md_rows.insert(1, separator);
        } else if !md_rows.is_empty() {
            md_rows.push(separator);
        }

        Ok(md_rows.join("\n"))
    }

    /// Converts the table to an HTML formatted string.
    ///
    /// # Returns
    ///
    /// A Result containing the HTML table string, or an error if text has not been extracted.
    ///
    /// # Errors
    ///
    /// Returns an error if `text_extracted` is false.
    pub fn to_html(&self) -> Result<String, &'static str> {
        if !self.text_extracted {
            return Err("Text has not been extracted. Call extract_text first.");
        }

        let rows = self.rows();
        if rows.is_empty() {
            return Ok("<table>\n</table>".to_string());
        }

        let mut html_parts: Vec<String> = Vec::new();
        html_parts.push("<table>".to_string());

        // Generate all rows
        for row in &rows {
            let cells_html: Vec<String> = row
                .cells
                .iter()
                .map(|cell| {
                    let text = cell.map(|c| c.text.as_str()).unwrap_or("");
                    format!("<td>{}</td>", escape_html_field(text))
                })
                .collect();
            html_parts.push(format!("<tr>{}</tr>", cells_html.join("")));
        }

        html_parts.push("</table>".to_string());
        Ok(html_parts.join("\n"))
    }
}

/// Filters edges by minimum length.
///
/// Removes edges shorter than the specified minimum length.
///
/// # Arguments
///
/// * `edges` - The edges to filter (modified in place).
/// * `min_len` - The minimum length threshold.
fn filter_edges_by_min_len(edges: &mut Vec<Edge>, min_len: OrderedFloat<f32>) {
    edges.retain(|edge| match edge.orientation {
        Orientation::Horizontal => (edge.x2 - edge.x1) >= min_len,
        Orientation::Vertical => (edge.y2 - edge.y1) >= min_len,
    });
}

/// Finds all intersections between horizontal and vertical edges.
///
/// # Arguments
///
/// * `edges` - A HashMap of edges by orientation.
/// * `intersection_x_tolerance` - X-tolerance for intersection detection.
/// * `intersection_y_tolerance` - Y-tolerance for intersection detection.
///
/// # Returns
///
/// A HashMap mapping intersection points to the edges that meet there.
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

/// Converts a slice of edges to a set of bounding box keys.
#[inline]
fn edges_to_set(edges: &[Edge]) -> HashSet<BboxKey> {
    edges.iter().map(|e| e.to_bbox_key()).collect()
}

/// Converts edge intersections into table cell bounding boxes.
///
/// Finds the smallest rectangular cells formed by the intersecting edges.
///
/// # Arguments
///
/// * `intersections` - The intersection points and their connecting edges.
///
/// # Returns
///
/// A vector of bounding boxes representing table cells.
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
            let set1 = edges_to_set(inter1.get(&Orientation::Vertical).unwrap());
            let set2 = edges_to_set(inter2.get(&Orientation::Vertical).unwrap());
            if !set1.is_disjoint(&set2) {
                return true;
            }
        }

        if p1.1 == p2.1 {
            let set1 = edges_to_set(inter1.get(&Orientation::Horizontal).unwrap());
            let set2 = edges_to_set(inter1.get(&Orientation::Horizontal).unwrap());
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

    (0..n_points).filter_map(find_smallest_cell).collect()
}

/// Extracts the four corner points of a bounding box.
///
/// # Arguments
///
/// * `bbox` - The bounding box.
///
/// # Returns
///
/// An array of the four corner points.
fn bbox_to_corners(bbox: &BboxKey) -> [Point; 4] {
    let (x1, y1, x2, y2) = *bbox;
    [(x1, y1), (x1, y2), (x2, y1), (x2, y2)]
}

/// Groups cells into separate tables based on connectivity.
///
/// Cells that share corners are grouped into the same table.
///
/// # Arguments
///
/// * `cells` - All detected cell bounding boxes.
///
/// # Returns
///
/// A vector of tables, each containing its cells' bounding boxes.
pub fn cells_to_tables(cells: &[BboxKey], include_single_cell: bool) -> Vec<Vec<BboxKey>> {
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

    if include_single_cell {
        tables
    } else {
        tables.into_iter().filter(|t| t.len() > 1).collect()
    }
}
/// Finds tables in PDF pages using edge detection.
pub(crate) struct TableFinder {
    /// The settings for table finding.
    settings: Rc<TfSettings>,
}

impl TableFinder {
    /// Creates a new TableFinder with the specified settings.
    ///
    /// # Arguments
    ///
    /// * `settings` - The table finder settings.
    ///
    /// # Returns
    ///
    /// A new TableFinder instance.
    pub(crate) fn new(settings: Rc<TfSettings>) -> Self {
        TableFinder {
            settings: settings.clone(),
        }
    }

    /// Extracts and processes edges from a PDF page.
    ///
    /// # Arguments
    ///
    /// * `page` - The PDF page to extract edges from.
    ///
    /// # Returns
    ///
    /// A HashMap of edges grouped by orientation.
    pub(crate) fn get_edges(&self, page: &Page) -> HashMap<Orientation, Vec<Edge>> {
        let settings = self.settings.as_ref();

        let objects_opt = page.objects.borrow();
        if objects_opt.is_none() {
            page.extract_objects();
        }
        let objects = objects_opt.as_ref().expect("Objects should be extracted");
        let mut edges_all = make_edges(objects, self.settings.clone());
        let mut v_edges = edges_all.remove(&Orientation::Vertical).unwrap_or_default();
        filter_edges_by_min_len(&mut v_edges, *settings.edge_min_length_prefilter);
        let mut h_edges = edges_all
            .remove(&Orientation::Horizontal)
            .unwrap_or_default();
        filter_edges_by_min_len(&mut h_edges, *settings.edge_min_length_prefilter);

        let edges_prefiltered = HashMap::from([
            (Orientation::Vertical, v_edges),
            (Orientation::Horizontal, h_edges),
        ]);
        let mut edges_merged = merge_edges(
            edges_prefiltered,
            *settings.snap_x_tolerance,
            *settings.snap_y_tolerance,
            *settings.join_x_tolerance,
            *settings.join_y_tolerance,
        );
        if let Some(h_edges) = edges_merged.get_mut(&Orientation::Horizontal) {
            filter_edges_by_min_len(h_edges, *settings.edge_min_length);
        }
        if let Some(v_edges) = edges_merged.get_mut(&Orientation::Vertical) {
            filter_edges_by_min_len(v_edges, *settings.edge_min_length);
        }
        edges_merged
    }
}

/// Finds all table cell bounding boxes in a PDF page.
///
/// # Arguments
///
/// * `pdf_page` - The PDF page to analyze.
/// * `tf_settings` - The table finder settings.
///
/// # Returns
///
/// A vector of bounding boxes for detected cells.
pub fn find_all_cells_bboxes(pdf_page: &Page, tf_settings: Rc<TfSettings>) -> Vec<BboxKey> {
    let table_finder = TableFinder::new(tf_settings.clone());
    let edges = table_finder.get_edges(pdf_page);
    let intersections = edges_to_intersections(
        &mut edges.clone(),
        *table_finder.settings.intersection_x_tolerance,
        *table_finder.settings.intersection_y_tolerance,
    );
    intersections_to_cells(intersections)
}

/// Creates Table objects from cell bounding boxes.
///
/// # Arguments
///
/// * `cells` - The cell bounding boxes.
/// * `extract_text` - Whether to extract text from cells.
/// * `pdf_page` - The PDF page (required if extract_text is true).
/// * `tf_settings` - Optional table finder settings.
///
/// # Returns
///
/// A vector of Table objects.
pub fn find_tables_from_cells(
    cells: &[BboxKey],
    extract_text: bool,
    pdf_page: Option<&Page>,
    tf_settings: Option<&TfSettings>,
) -> Vec<Table> {
    let include_single_cell = tf_settings.is_some_and(|s| s.include_single_cell);
    let need_strip = tf_settings.is_none_or(|s| s.text_settings.need_strip);
    let tables_bbox = cells_to_tables(cells, include_single_cell);

    let objects_guard = if extract_text {
        let page = match pdf_page {
            Some(p) => p,
            None => panic!("Page must be provided when extract_text is true"),
        };
        if page.objects.borrow().is_none() {
            page.extract_objects();
        }
        Some(page.objects.borrow())
    } else {
        None
    };
    let chars: Option<&[Char]> = objects_guard
        .as_ref()
        .map(|g| &g.as_ref().unwrap().chars[..]);
    let we_settings = tf_settings.map(|s| &s.text_settings);
    tables_bbox
        .iter()
        .map(|table_cells_bbox| {
            Table::new(
                0,
                table_cells_bbox,
                extract_text,
                chars,
                we_settings,
                need_strip,
            )
        })
        .collect()
}
/// Finds all tables in a PDF page.
///
/// This is the main entry point for table detection. It extracts edges,
/// finds intersections, builds cells, and groups them into tables.
///
/// # Arguments
///
/// * `pdf_page` - The PDF page to analyze.
/// * `tf_settings` - The table finder settings.
/// * `extract_text` - Whether to extract text content from cells.
///
/// # Returns
///
/// A vector of Table objects found in the page.
pub fn find_tables(pdf_page: &Page, tf_settings: Rc<TfSettings>, extract_text: bool) -> Vec<Table> {
    let cells = find_all_cells_bboxes(pdf_page, tf_settings.clone());
    find_tables_from_cells(&cells, extract_text, Some(pdf_page), Some(&tf_settings))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ordered_float::OrderedFloat;

    fn of(v: f32) -> OrderedFloat<f32> {
        OrderedFloat(v)
    }

    #[test]
    fn test_get_axis_value() {
        let bbox: BboxKey = (of(1.0), of(2.0), of(3.0), of(4.0));
        assert_eq!(get_axis_value(&bbox, 0), of(1.0)); // x1
        assert_eq!(get_axis_value(&bbox, 1), of(2.0)); // y1
        assert_eq!(get_axis_value(&bbox, 2), of(3.0)); // x2
        assert_eq!(get_axis_value(&bbox, 3), of(4.0)); // y2
    }

    #[test]
    #[should_panic(expected = "Invalid axis")]
    fn test_get_axis_value_invalid() {
        let bbox: BboxKey = (of(1.0), of(2.0), of(3.0), of(4.0));
        get_axis_value(&bbox, 4);
    }

    #[test]
    fn test_bbox_to_corners() {
        let bbox: BboxKey = (of(0.0), of(0.0), of(10.0), of(20.0));
        let corners = bbox_to_corners(&bbox);
        assert_eq!(corners[0], (of(0.0), of(0.0))); // top-left
        assert_eq!(corners[1], (of(0.0), of(20.0))); // bottom-left
        assert_eq!(corners[2], (of(10.0), of(0.0))); // top-right
        assert_eq!(corners[3], (of(10.0), of(20.0))); // bottom-right
    }

    #[test]
    fn test_cells_to_tables_single_table() {
        // Create a 2x2 table (4 cells sharing corners)
        let cells: Vec<BboxKey> = vec![
            (of(0.0), of(0.0), of(10.0), of(10.0)),
            (of(10.0), of(0.0), of(20.0), of(10.0)),
            (of(0.0), of(10.0), of(10.0), of(20.0)),
            (of(10.0), of(10.0), of(20.0), of(20.0)),
        ];
        let tables = cells_to_tables(&cells, false);
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].len(), 4);
    }

    #[test]
    fn test_cells_to_tables_two_separate_tables() {
        // Create two separate tables
        let cells: Vec<BboxKey> = vec![
            // Table 1 (2 cells)
            (of(0.0), of(0.0), of(10.0), of(10.0)),
            (of(10.0), of(0.0), of(20.0), of(10.0)),
            // Table 2 (2 cells, far away from table 1)
            (of(100.0), of(100.0), of(110.0), of(110.0)),
            (of(110.0), of(100.0), of(120.0), of(110.0)),
        ];
        let tables = cells_to_tables(&cells, false);
        assert_eq!(tables.len(), 2);
    }

    #[test]
    fn test_cells_to_tables_single_cell_excluded() {
        // A single cell should not form a table (needs at least 2 cells) when include_single_cell is false
        let cells: Vec<BboxKey> = vec![(of(0.0), of(0.0), of(10.0), of(10.0))];
        let tables = cells_to_tables(&cells, false);
        assert_eq!(tables.len(), 0);
    }

    #[test]
    fn test_cells_to_tables_single_cell_included() {
        // A single cell should form a table when include_single_cell is true
        let cells: Vec<BboxKey> = vec![(of(0.0), of(0.0), of(10.0), of(10.0))];
        let tables = cells_to_tables(&cells, true);
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].len(), 1);
    }

    #[test]
    fn test_cells_to_tables_empty() {
        let cells: Vec<BboxKey> = vec![];
        let tables = cells_to_tables(&cells, false);
        assert_eq!(tables.len(), 0);
    }

    #[test]
    fn test_cell_group_new() {
        let cell1 = TableCell {
            text: "A".to_string(),
            bbox: (of(0.0), of(0.0), of(10.0), of(10.0)),
        };
        let cell2 = TableCell {
            text: "B".to_string(),
            bbox: (of(10.0), of(0.0), of(20.0), of(10.0)),
        };
        let cells: Vec<Option<&TableCell>> = vec![Some(&cell1), None, Some(&cell2)];
        let group = CellGroup::new(cells);

        assert_eq!(group.cells.len(), 3);
        assert!(group.cells[0].is_some());
        assert!(group.cells[1].is_none());
        assert!(group.cells[2].is_some());
        // Bbox should encompass both cells
        assert_eq!(group.bbox.0, of(0.0)); // min x1
        assert_eq!(group.bbox.2, of(20.0)); // max x2
    }

    #[test]
    fn test_table_rows() {
        // Create a 2x2 table
        let cells = vec![
            TableCell {
                text: "A".to_string(),
                bbox: (of(0.0), of(0.0), of(10.0), of(10.0)),
            },
            TableCell {
                text: "B".to_string(),
                bbox: (of(10.0), of(0.0), of(20.0), of(10.0)),
            },
            TableCell {
                text: "C".to_string(),
                bbox: (of(0.0), of(10.0), of(10.0), of(20.0)),
            },
            TableCell {
                text: "D".to_string(),
                bbox: (of(10.0), of(10.0), of(20.0), of(20.0)),
            },
        ];
        let table = Table {
            cells,
            bbox: (of(0.0), of(0.0), of(20.0), of(20.0)),
            page_index: 0,
            text_extracted: true,
        };

        let rows = table.rows();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].cells.len(), 2);
        assert_eq!(rows[1].cells.len(), 2);
    }

    #[test]
    fn test_table_columns() {
        // Create a 2x2 table
        let cells = vec![
            TableCell {
                text: "A".to_string(),
                bbox: (of(0.0), of(0.0), of(10.0), of(10.0)),
            },
            TableCell {
                text: "B".to_string(),
                bbox: (of(10.0), of(0.0), of(20.0), of(10.0)),
            },
            TableCell {
                text: "C".to_string(),
                bbox: (of(0.0), of(10.0), of(10.0), of(20.0)),
            },
            TableCell {
                text: "D".to_string(),
                bbox: (of(10.0), of(10.0), of(20.0), of(20.0)),
            },
        ];
        let table = Table {
            cells,
            bbox: (of(0.0), of(0.0), of(20.0), of(20.0)),
            page_index: 0,
            text_extracted: true,
        };

        let cols = table.columns();
        assert_eq!(cols.len(), 2);
        assert_eq!(cols[0].cells.len(), 2);
        assert_eq!(cols[1].cells.len(), 2);
    }

    #[test]
    fn test_char_in_bbox() {
        let char = Char {
            unicode_char: Some("A".to_string()),
            bbox: (of(5.0), of(5.0), of(8.0), of(8.0)),
            rotation_degrees: of(0.0),
            upright: true,
        };
        let bbox: BboxKey = (of(0.0), of(0.0), of(10.0), of(10.0));

        // Char center is (6.5, 6.5), which is inside the bbox
        assert!(Table::char_in_bbox(&char, &bbox));
    }

    #[test]
    fn test_char_not_in_bbox() {
        let char = Char {
            unicode_char: Some("A".to_string()),
            bbox: (of(15.0), of(15.0), of(18.0), of(18.0)),
            rotation_degrees: of(0.0),
            upright: true,
        };
        let bbox: BboxKey = (of(0.0), of(0.0), of(10.0), of(10.0));

        // Char center is (16.5, 16.5), which is outside the bbox
        assert!(!Table::char_in_bbox(&char, &bbox));
    }

    #[test]
    fn test_filter_edges_by_min_len() {
        use crate::edges::Edge;
        use pdfium_render::prelude::PdfColor;

        let mut edges = vec![
            Edge {
                orientation: Orientation::Horizontal,
                x1: of(0.0),
                y1: of(0.0),
                x2: of(5.0), // length = 5
                y2: of(0.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Horizontal,
                x1: of(0.0),
                y1: of(10.0),
                x2: of(15.0), // length = 15
                y2: of(10.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Vertical,
                x1: of(0.0),
                y1: of(0.0),
                x2: of(0.0),
                y2: of(3.0), // length = 3
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
        ];

        filter_edges_by_min_len(&mut edges, of(10.0));
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].x2, of(15.0)); // Only the long horizontal edge remains
    }

    #[test]
    fn test_escape_csv_field_simple() {
        assert_eq!(escape_csv_field("hello"), "hello");
        assert_eq!(escape_csv_field("world"), "world");
    }

    #[test]
    fn test_escape_csv_field_with_comma() {
        assert_eq!(escape_csv_field("hello,world"), "\"hello,world\"");
    }

    #[test]
    fn test_escape_csv_field_with_quotes() {
        assert_eq!(escape_csv_field("say \"hi\""), "\"say \"\"hi\"\"\"");
    }

    #[test]
    fn test_escape_csv_field_with_newline() {
        assert_eq!(escape_csv_field("line1\nline2"), "\"line1\nline2\"");
        assert_eq!(escape_csv_field("line1\r\nline2"), "\"line1\r\nline2\"");
    }

    #[test]
    fn test_to_csv_basic() {
        // Create a 2x2 table with text
        let cells = vec![
            TableCell {
                text: "A".to_string(),
                bbox: (of(0.0), of(0.0), of(10.0), of(10.0)),
            },
            TableCell {
                text: "B".to_string(),
                bbox: (of(10.0), of(0.0), of(20.0), of(10.0)),
            },
            TableCell {
                text: "C".to_string(),
                bbox: (of(0.0), of(10.0), of(10.0), of(20.0)),
            },
            TableCell {
                text: "D".to_string(),
                bbox: (of(10.0), of(10.0), of(20.0), of(20.0)),
            },
        ];
        let table = Table {
            cells,
            bbox: (of(0.0), of(0.0), of(20.0), of(20.0)),
            page_index: 0,
            text_extracted: true,
        };

        let csv = table.to_csv().unwrap();
        assert_eq!(csv, "A,B\nC,D");
    }

    #[test]
    fn test_to_csv_with_empty_cells() {
        // Create a table with some empty cells
        let cells = vec![
            TableCell {
                text: "abc ".to_string(),
                bbox: (of(0.0), of(0.0), of(10.0), of(10.0)),
            },
            TableCell {
                text: "q".to_string(),
                bbox: (of(10.0), of(0.0), of(20.0), of(10.0)),
            },
            TableCell {
                text: "".to_string(),
                bbox: (of(0.0), of(10.0), of(10.0), of(20.0)),
            },
            TableCell {
                text: "w".to_string(),
                bbox: (of(10.0), of(10.0), of(20.0), of(20.0)),
            },
            TableCell {
                text: "1 ".to_string(),
                bbox: (of(0.0), of(20.0), of(10.0), of(30.0)),
            },
            TableCell {
                text: "2".to_string(),
                bbox: (of(10.0), of(20.0), of(20.0), of(30.0)),
            },
            TableCell {
                text: "3 ".to_string(),
                bbox: (of(0.0), of(30.0), of(10.0), of(40.0)),
            },
            TableCell {
                text: "4 ".to_string(),
                bbox: (of(10.0), of(30.0), of(20.0), of(40.0)),
            },
        ];
        let table = Table {
            cells,
            bbox: (of(0.0), of(0.0), of(20.0), of(40.0)),
            page_index: 0,
            text_extracted: true,
        };

        let csv = table.to_csv().unwrap();
        assert_eq!(csv, "abc ,q\n,w\n1 ,2\n3 ,4 ");
    }

    #[test]
    fn test_to_csv_without_text_extracted() {
        let cells = vec![TableCell {
            text: "".to_string(),
            bbox: (of(0.0), of(0.0), of(10.0), of(10.0)),
        }];
        let table = Table {
            cells,
            bbox: (of(0.0), of(0.0), of(10.0), of(10.0)),
            page_index: 0,
            text_extracted: false,
        };

        let result = table.to_csv();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Text has not been extracted. Call extract_text first."
        );
    }

    #[test]
    fn test_to_csv_with_special_chars() {
        // Create a table with special CSV characters
        let cells = vec![
            TableCell {
                text: "hello,world".to_string(),
                bbox: (of(0.0), of(0.0), of(10.0), of(10.0)),
            },
            TableCell {
                text: "say \"hi\"".to_string(),
                bbox: (of(10.0), of(0.0), of(20.0), of(10.0)),
            },
        ];
        let table = Table {
            cells,
            bbox: (of(0.0), of(0.0), of(20.0), of(10.0)),
            page_index: 0,
            text_extracted: true,
        };

        let csv = table.to_csv().unwrap();
        assert_eq!(csv, "\"hello,world\",\"say \"\"hi\"\"\"");
    }

    #[test]
    fn test_escape_markdown_field_simple() {
        assert_eq!(escape_markdown_field("hello"), "hello");
        assert_eq!(escape_markdown_field("world"), "world");
    }

    #[test]
    fn test_escape_markdown_field_with_pipe() {
        assert_eq!(escape_markdown_field("a|b"), "a\\|b");
        assert_eq!(escape_markdown_field("|start"), "\\|start");
        assert_eq!(escape_markdown_field("end|"), "end\\|");
    }

    #[test]
    fn test_escape_markdown_field_with_newline() {
        assert_eq!(escape_markdown_field("line1\nline2"), "line1<br>line2");
        assert_eq!(escape_markdown_field("line1\r\nline2"), "line1<br>line2");
    }

    #[test]
    fn test_to_markdown_basic() {
        // Create a 2x2 table with text
        let cells = vec![
            TableCell {
                text: "A".to_string(),
                bbox: (of(0.0), of(0.0), of(10.0), of(10.0)),
            },
            TableCell {
                text: "B".to_string(),
                bbox: (of(10.0), of(0.0), of(20.0), of(10.0)),
            },
            TableCell {
                text: "C".to_string(),
                bbox: (of(0.0), of(10.0), of(10.0), of(20.0)),
            },
            TableCell {
                text: "D".to_string(),
                bbox: (of(10.0), of(10.0), of(20.0), of(20.0)),
            },
        ];
        let table = Table {
            cells,
            bbox: (of(0.0), of(0.0), of(20.0), of(20.0)),
            page_index: 0,
            text_extracted: true,
        };

        let markdown = table.to_markdown().unwrap();
        assert_eq!(markdown, "| A | B |\n| --- | --- |\n| C | D |");
    }

    #[test]
    fn test_to_markdown_with_empty_cells() {
        // Create a table with some empty cells
        let cells = vec![
            TableCell {
                text: "abc ".to_string(),
                bbox: (of(0.0), of(0.0), of(10.0), of(10.0)),
            },
            TableCell {
                text: "q".to_string(),
                bbox: (of(10.0), of(0.0), of(20.0), of(10.0)),
            },
            TableCell {
                text: "".to_string(),
                bbox: (of(0.0), of(10.0), of(10.0), of(20.0)),
            },
            TableCell {
                text: "w".to_string(),
                bbox: (of(10.0), of(10.0), of(20.0), of(20.0)),
            },
            TableCell {
                text: "1 ".to_string(),
                bbox: (of(0.0), of(20.0), of(10.0), of(30.0)),
            },
            TableCell {
                text: "2".to_string(),
                bbox: (of(10.0), of(20.0), of(20.0), of(30.0)),
            },
            TableCell {
                text: "3 ".to_string(),
                bbox: (of(0.0), of(30.0), of(10.0), of(40.0)),
            },
            TableCell {
                text: "4 ".to_string(),
                bbox: (of(10.0), of(30.0), of(20.0), of(40.0)),
            },
        ];
        let table = Table {
            cells,
            bbox: (of(0.0), of(0.0), of(20.0), of(40.0)),
            page_index: 0,
            text_extracted: true,
        };

        let markdown = table.to_markdown().unwrap();
        assert_eq!(
            markdown,
            "| abc  | q |\n| --- | --- |\n|  | w |\n| 1  | 2 |\n| 3  | 4  |"
        );
    }

    #[test]
    fn test_to_markdown_without_text_extracted() {
        let cells = vec![TableCell {
            text: "".to_string(),
            bbox: (of(0.0), of(0.0), of(10.0), of(10.0)),
        }];
        let table = Table {
            cells,
            bbox: (of(0.0), of(0.0), of(10.0), of(10.0)),
            page_index: 0,
            text_extracted: false,
        };

        let result = table.to_markdown();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Text has not been extracted. Call extract_text first."
        );
    }

    #[test]
    fn test_to_markdown_with_special_chars() {
        // Create a table with special Markdown characters
        let cells = vec![
            TableCell {
                text: "hello|world".to_string(),
                bbox: (of(0.0), of(0.0), of(10.0), of(10.0)),
            },
            TableCell {
                text: "line1\nline2".to_string(),
                bbox: (of(10.0), of(0.0), of(20.0), of(10.0)),
            },
        ];
        let table = Table {
            cells,
            bbox: (of(0.0), of(0.0), of(20.0), of(10.0)),
            page_index: 0,
            text_extracted: true,
        };

        let markdown = table.to_markdown().unwrap();
        assert_eq!(
            markdown,
            "| hello\\|world | line1<br>line2 |\n| --- | --- |"
        );
    }

    #[test]
    fn test_to_markdown_single_row() {
        // Create a table with only one row
        let cells = vec![
            TableCell {
                text: "Header1".to_string(),
                bbox: (of(0.0), of(0.0), of(10.0), of(10.0)),
            },
            TableCell {
                text: "Header2".to_string(),
                bbox: (of(10.0), of(0.0), of(20.0), of(10.0)),
            },
        ];
        let table = Table {
            cells,
            bbox: (of(0.0), of(0.0), of(20.0), of(10.0)),
            page_index: 0,
            text_extracted: true,
        };

        let markdown = table.to_markdown().unwrap();
        assert_eq!(markdown, "| Header1 | Header2 |\n| --- | --- |");
    }

    #[test]
    fn test_escape_html_field_simple() {
        assert_eq!(escape_html_field("hello"), "hello");
        assert_eq!(escape_html_field("world"), "world");
    }

    #[test]
    fn test_escape_html_field_with_ampersand() {
        assert_eq!(escape_html_field("a & b"), "a &amp; b");
        assert_eq!(escape_html_field("&start"), "&amp;start");
    }

    #[test]
    fn test_escape_html_field_with_angle_brackets() {
        assert_eq!(escape_html_field("<tag>"), "&lt;tag&gt;");
        assert_eq!(escape_html_field("a < b > c"), "a &lt; b &gt; c");
    }

    #[test]
    fn test_escape_html_field_with_quotes() {
        assert_eq!(escape_html_field("say \"hi\""), "say &quot;hi&quot;");
    }

    #[test]
    fn test_escape_html_field_with_newline() {
        assert_eq!(escape_html_field("line1\nline2"), "line1<br>line2");
        assert_eq!(escape_html_field("line1\r\nline2"), "line1<br>line2");
    }

    #[test]
    fn test_escape_html_field_complex() {
        assert_eq!(
            escape_html_field("<a href=\"test\">link & text</a>"),
            "&lt;a href=&quot;test&quot;&gt;link &amp; text&lt;/a&gt;"
        );
    }

    #[test]
    fn test_to_html_basic() {
        // Create a 2x2 table with text
        let cells = vec![
            TableCell {
                text: "A".to_string(),
                bbox: (of(0.0), of(0.0), of(10.0), of(10.0)),
            },
            TableCell {
                text: "B".to_string(),
                bbox: (of(10.0), of(0.0), of(20.0), of(10.0)),
            },
            TableCell {
                text: "C".to_string(),
                bbox: (of(0.0), of(10.0), of(10.0), of(20.0)),
            },
            TableCell {
                text: "D".to_string(),
                bbox: (of(10.0), of(10.0), of(20.0), of(20.0)),
            },
        ];
        let table = Table {
            cells,
            bbox: (of(0.0), of(0.0), of(20.0), of(20.0)),
            page_index: 0,
            text_extracted: true,
        };

        let html = table.to_html().unwrap();
        assert_eq!(
            html,
            "<table>\n<tr><td>A</td><td>B</td></tr>\n<tr><td>C</td><td>D</td></tr>\n</table>"
        );
    }

    #[test]
    fn test_to_html_with_empty_cells() {
        // Create a table with some empty cells
        let cells = vec![
            TableCell {
                text: "abc ".to_string(),
                bbox: (of(0.0), of(0.0), of(10.0), of(10.0)),
            },
            TableCell {
                text: "q".to_string(),
                bbox: (of(10.0), of(0.0), of(20.0), of(10.0)),
            },
            TableCell {
                text: "".to_string(),
                bbox: (of(0.0), of(10.0), of(10.0), of(20.0)),
            },
            TableCell {
                text: "w".to_string(),
                bbox: (of(10.0), of(10.0), of(20.0), of(20.0)),
            },
            TableCell {
                text: "1 ".to_string(),
                bbox: (of(0.0), of(20.0), of(10.0), of(30.0)),
            },
            TableCell {
                text: "2".to_string(),
                bbox: (of(10.0), of(20.0), of(20.0), of(30.0)),
            },
            TableCell {
                text: "3 ".to_string(),
                bbox: (of(0.0), of(30.0), of(10.0), of(40.0)),
            },
            TableCell {
                text: "4 ".to_string(),
                bbox: (of(10.0), of(30.0), of(20.0), of(40.0)),
            },
        ];
        let table = Table {
            cells,
            bbox: (of(0.0), of(0.0), of(20.0), of(40.0)),
            page_index: 0,
            text_extracted: true,
        };

        let html = table.to_html().unwrap();
        assert_eq!(
            html,
            "<table>\n<tr><td>abc </td><td>q</td></tr>\n<tr><td></td><td>w</td></tr>\n<tr><td>1 </td><td>2</td></tr>\n<tr><td>3 </td><td>4 </td></tr>\n</table>"
        );
    }

    #[test]
    fn test_to_html_without_text_extracted() {
        let cells = vec![TableCell {
            text: "".to_string(),
            bbox: (of(0.0), of(0.0), of(10.0), of(10.0)),
        }];
        let table = Table {
            cells,
            bbox: (of(0.0), of(0.0), of(10.0), of(10.0)),
            page_index: 0,
            text_extracted: false,
        };

        let result = table.to_html();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Text has not been extracted. Call extract_text first."
        );
    }

    #[test]
    fn test_to_html_with_special_chars() {
        // Create a table with special HTML characters
        let cells = vec![
            TableCell {
                text: "<script>alert('xss')</script>".to_string(),
                bbox: (of(0.0), of(0.0), of(10.0), of(10.0)),
            },
            TableCell {
                text: "a & b".to_string(),
                bbox: (of(10.0), of(0.0), of(20.0), of(10.0)),
            },
        ];
        let table = Table {
            cells,
            bbox: (of(0.0), of(0.0), of(20.0), of(10.0)),
            page_index: 0,
            text_extracted: true,
        };

        let html = table.to_html().unwrap();
        assert_eq!(
            html,
            "<table>\n<tr><td>&lt;script&gt;alert('xss')&lt;/script&gt;</td><td>a &amp; b</td></tr>\n</table>"
        );
    }

    #[test]
    fn test_to_html_empty_table() {
        let table = Table {
            cells: vec![],
            bbox: (of(0.0), of(0.0), of(0.0), of(0.0)),
            page_index: 0,
            text_extracted: true,
        };

        let html = table.to_html().unwrap();
        assert_eq!(html, "<table>\n</table>");
    }

    #[test]
    fn test_to_html_single_row() {
        // Create a table with only one row
        let cells = vec![
            TableCell {
                text: "Header1".to_string(),
                bbox: (of(0.0), of(0.0), of(10.0), of(10.0)),
            },
            TableCell {
                text: "Header2".to_string(),
                bbox: (of(10.0), of(0.0), of(20.0), of(10.0)),
            },
        ];
        let table = Table {
            cells,
            bbox: (of(0.0), of(0.0), of(20.0), of(10.0)),
            page_index: 0,
            text_extracted: true,
        };

        let html = table.to_html().unwrap();
        assert_eq!(
            html,
            "<table>\n<tr><td>Header1</td><td>Header2</td></tr>\n</table>"
        );
    }
}
