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

    /// Converts to an owned PyCellGroup for Python.
    pub fn to_owned(&self) -> PyCellGroup {
        PyCellGroup {
            cells: self.cells.iter().map(|c| c.cloned()).collect(),
            bbox: self.bbox,
        }
    }
}

/// An owned version of CellGroup for Python interop.
#[pyclass(name = "CellGroup")]
#[derive(Debug, Clone)]
pub struct PyCellGroup {
    /// The cells in this group, with `None` for empty positions.
    #[pyo3(get)]
    pub cells: Vec<Option<TableCell>>,
    /// The bounding box of the entire group.
    pub bbox: BboxKey,
}

#[pymethods]
impl PyCellGroup {
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

/// Value of one grid slot: either text (top-left of a cell) or merge info.
/// `merged_left` / `merged_top` indicate whether the spanning cell started left or above.
#[derive(Debug, Clone, PartialEq)]
pub struct TableCellValue {
    pub text: Option<String>,
    pub merged_left: bool,
    pub merged_top: bool,
}

/// Python-exposed TableCellValue for to_list().
#[pyclass(name = "TableCellValue")]
#[derive(Debug, Clone)]
pub struct PyTableCellValue {
    #[pyo3(get)]
    pub text: Option<String>,
    #[pyo3(get)]
    pub merged_left: bool,
    #[pyo3(get)]
    pub merged_top: bool,
}

fn py_table_cell_value_repr(text: &Option<String>, merged_left: bool, merged_top: bool) -> String {
    let text_repr = match text {
        None => "None".to_string(),
        Some(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
    };
    let left_str = if merged_left { "True" } else { "False" };
    let top_str = if merged_top { "True" } else { "False" };
    format!("({}, {}, {})", text_repr, left_str, top_str)
}

#[pymethods]
impl PyTableCellValue {
    /// Return repr as "(text, merged_left, merged_top)". Text is None or double-quoted (escaped); booleans are True/False.
    fn __repr__(&self) -> String {
        py_table_cell_value_repr(&self.text, self.merged_left, self.merged_top)
    }
}

/// Returns true if point (x, y) is inside bbox (x1 <= x < x2, y1 <= y < y2).
fn point_in_bbox(x: OrderedFloat<f32>, y: OrderedFloat<f32>, bbox: &BboxKey) -> bool {
    x >= bbox.0 && x < bbox.2 && y >= bbox.1 && y < bbox.3
}

/// Tolerance for float comparison when determining merge direction (same slot vs left/above).
const MERGE_DIRECTION_TOLERANCE: f32 = 0.001;

/// Returns true if two OrderedFloat values are equal within tolerance (for merge direction).
fn float_eq(a: OrderedFloat<f32>, b: OrderedFloat<f32>) -> bool {
    (a - b).abs() < MERGE_DIRECTION_TOLERANCE
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

    /// Get rows
    /// Returns a vector of rows, where each row is a vector of cells or None
    #[getter]
    #[pyo3(name = "rows")]
    fn py_rows(&self) -> Vec<PyCellGroup> {
        self.rows().iter().map(|r| r.to_owned()).collect()
    }

    /// Get columns
    /// Returns a vector of columns, where each column is a vector of cells or None
    #[getter]
    #[pyo3(name = "columns")]
    fn py_columns(&self) -> Vec<PyCellGroup> {
        self.columns().iter().map(|c| c.to_owned()).collect()
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

    /// Converts the table to a list of rows; each element is a TableCellValue with text and merge flags.
    ///
    /// Each inner list is one row. TableCellValue has `text` (None when merged), `merged_left`,
    /// and `merged_top` so you can tell if the slot is merged with the cell to the left or above.
    ///
    /// # Errors
    ///
    /// Returns a PyValueError if text has not been extracted.
    #[pyo3(name = "to_list")]
    fn py_to_list(&self) -> PyResult<Vec<Vec<PyTableCellValue>>> {
        self.to_vec()
            .map(|vecs| {
                vecs.into_iter()
                    .map(|row| {
                        row.into_iter()
                            .map(|v| PyTableCellValue {
                                text: v.text,
                                merged_left: v.merged_left,
                                merged_top: v.merged_top,
                            })
                            .collect()
                    })
                    .collect()
            })
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

    /// Returns `true` when the gap between two consecutive words (in reading direction) exceeds
    /// the relevant tolerance, indicating that a space should be inserted when joining them.
    ///
    /// The gap direction and tolerance axis are chosen from `next`'s rotation:
    /// - LTR / RTL (horizontal): compares horizontal bbox edges, uses `x_tol`.
    /// - Top-to-bottom / bottom-to-top (vertical): compares vertical bbox edges, uses `y_tol`.
    ///
    /// **Assumption**: `prev` and `next` share the same reading direction. When a cell contains
    /// mixed-rotation text the result is undefined and this function uses `next`'s rotation only.
    #[inline]
    fn word_gap_requires_space(prev: &Word, next: &Word, x_tol: f32, y_tol: f32) -> bool {
        let r = next.rotation_degrees;
        let gap = if rotation_is_ltr(r) {
            // horizontal LTR: gap = next left − prev right
            next.bbox.0 - prev.bbox.2
        } else if r >= OrderedFloat(45.0f32) && r < OrderedFloat(135.0f32) {
            // vertical top-to-bottom: gap = next top − prev bottom
            next.bbox.1 - prev.bbox.3
        } else if r >= OrderedFloat(135.0f32) && r < OrderedFloat(225.0f32) {
            // horizontal RTL: gap = prev left − next right
            prev.bbox.0 - next.bbox.2
        } else {
            // vertical bottom-to-top: gap = prev top − next bottom
            prev.bbox.1 - next.bbox.3
        };
        let tol = if rotation_is_horizontal(r) {
            OrderedFloat(x_tol)
        } else {
            OrderedFloat(y_tol)
        };
        gap > tol
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
        let x_tol = word_settings.x_tolerance.into_inner();
        let y_tol = word_settings.y_tolerance.into_inner();

        for cell in &mut self.cells {
            let cell_chars: Vec<Char> = chars
                .iter()
                .filter(|char| Self::char_in_bbox(char, &cell.bbox))
                .cloned()
                .collect();

            if !cell_chars.is_empty() {
                let words = word_extractor.extract_words(&cell_chars);
                let mut text = String::new();
                for (i, w) in words.iter().enumerate() {
                    if i > 0 {
                        let prev = &words[i - 1];
                        if Self::word_gap_requires_space(prev, w, x_tol, y_tol) {
                            text.push(' ');
                        }
                    }
                    text.push_str(&w.text.replace("\r\n", "\n").replace('\r', "\n"));
                }
                if need_strip {
                    text = text.trim().to_string();
                }
                cell.text = text;
            }
        }
        self.text_extracted = true;
    }

    /// Converts the table to a vector of rows; each cell has text and merge direction flags.
    ///
    /// For each empty (merged) slot we find the covering cell by scanning `self.cells`; for very
    /// large tables with many merged cells, a spatial index could be added later.
    ///
    /// # Returns
    ///
    /// A Result containing `Vec<Vec<TableCellValue>>`. Each slot has `text` (None when merged),
    /// `merged_left` (cell spans from left), and `merged_top` (cell spans from above).
    ///
    /// # Errors
    ///
    /// Returns an error if `text_extracted` is false.
    pub fn to_vec(&self) -> Result<Vec<Vec<TableCellValue>>, &'static str> {
        if !self.text_extracted {
            return Err("Text has not been extracted. Call extract_text first.");
        }

        let rows = self.rows();
        // Column positions xs match rows() column order (one x per column index).
        let xs: Vec<OrderedFloat<f32>> = self
            .cells
            .iter()
            .map(|c| c.bbox.0)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let row_ys: Vec<OrderedFloat<f32>> = rows.iter().map(|r| r.bbox.1).collect();

        let vecs: Vec<Vec<TableCellValue>> = rows
            .iter()
            .enumerate()
            .map(|(ri, row)| {
                row.cells
                    .iter()
                    .enumerate()
                    .map(|(ci, cell)| {
                        if let Some(c) = cell {
                            TableCellValue {
                                text: Some(c.text.clone()),
                                merged_left: false,
                                merged_top: false,
                            }
                        } else {
                            let x = xs[ci];
                            let y = row_ys[ri];
                            // Each slot is assumed covered by at most one cell; if multiple overlap we use the first match.
                            let covering = self.cells.iter().find(|c| point_in_bbox(x, y, &c.bbox));
                            let (merged_left, merged_top) = match covering {
                                Some(c) => (float_eq(c.bbox.1, y), float_eq(c.bbox.0, x)),
                                None => (false, false),
                            };
                            TableCellValue {
                                text: None,
                                merged_left,
                                merged_top,
                            }
                        }
                    })
                    .collect()
            })
            .collect();

        Ok(vecs)
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

/// Returns `true` when a rectangle qualifies as a background fill for adjacency checks.
///
/// A qualifying rect has a non-NONE fill mode and both dimensions ≥ the respective
/// snap tolerances.  Narrow rects (width < snap_x_tol or height < snap_y_tol) are
/// themselves treated as edges by `make_edges` and must not be considered backgrounds.
///
/// **Coupling note**: the size thresholds intentionally mirror the same `snap_x_tol` /
/// `snap_y_tol` values used by `make_edges` to promote thin rects into edges.  If the
/// edge-promotion thresholds in `make_edges` are ever changed to a different constant,
/// this function must be updated in tandem to avoid misclassifying thin rects as
/// background fills (or vice-versa).
#[inline]
fn is_qualifying_rect(r: &Rect, snap_x_tol: f32, snap_y_tol: f32) -> bool {
    FillMode::from(r.fill_mode) != FillMode::NONE
        && (r.bbox.2 - r.bbox.0).into_inner().abs() >= snap_x_tol
        && (r.bbox.3 - r.bbox.1).into_inner().abs() >= snap_y_tol
}

/// Returns the RGB fill color of the largest qualifying rect that lies immediately
/// adjacent to one side of `edge`.
///
/// For a **horizontal** edge at `y = Y` spanning `[x1, x2]`:
/// - `first_side = true`  → look for rects whose **bottom** (`bbox.3`) is near `Y`
///   (the rect sits above the edge in downward-y screen coordinates).
/// - `first_side = false` → look for rects whose **top** (`bbox.1`) is near `Y`
///   (the rect sits below the edge).
///
/// For a **vertical** edge at `x = X` spanning `[y1, y2]`:
/// - `first_side = true`  → look for rects whose **right** edge (`bbox.2`) is near `X`.
/// - `first_side = false` → look for rects whose **left** edge (`bbox.0`) is near `X`.
///
/// Adjacency criteria:
/// - Perpendicular distance ≤ `snap_perp_tol` (snap_y_tol for H, snap_x_tol for V).
/// - Parallel overlap with the edge > `snap_par_tol` (snap_x_tol for H, snap_y_tol for V).
///
/// Returns `None` if no qualifying rect is found on that side.
fn find_adjacent_rect_color(
    rects: &[Rect],
    edge: &Edge,
    first_side: bool,
    snap_x_tol: f32,
    snap_y_tol: f32,
) -> Option<(u8, u8, u8)> {
    let mut best_area = 0.0f32;
    let mut best_color: Option<(u8, u8, u8)> = None;

    for r in rects {
        if !is_qualifying_rect(r, snap_x_tol, snap_y_tol) {
            continue;
        }

        let (perp_dist, par_overlap) = match edge.orientation {
            Orientation::Horizontal => {
                let y = edge.y1.into_inner();
                let boundary = if first_side {
                    r.bbox.3.into_inner() // bottom of rect → rect is above the edge
                } else {
                    r.bbox.1.into_inner() // top of rect → rect is below the edge
                };
                let x_overlap = r.bbox.2.into_inner().min(edge.x2.into_inner())
                    - r.bbox.0.into_inner().max(edge.x1.into_inner());
                ((boundary - y).abs(), x_overlap)
            }
            Orientation::Vertical => {
                let x = edge.x1.into_inner();
                let boundary = if first_side {
                    r.bbox.2.into_inner() // right edge of rect → rect is left of the edge
                } else {
                    r.bbox.0.into_inner() // left edge of rect → rect is right of the edge
                };
                let y_overlap = r.bbox.3.into_inner().min(edge.y2.into_inner())
                    - r.bbox.1.into_inner().max(edge.y1.into_inner());
                ((boundary - x).abs(), y_overlap)
            }
        };

        let snap_perp = match edge.orientation {
            Orientation::Horizontal => snap_y_tol,
            Orientation::Vertical => snap_x_tol,
        };
        let snap_par = match edge.orientation {
            Orientation::Horizontal => snap_x_tol,
            Orientation::Vertical => snap_y_tol,
        };

        if perp_dist > snap_perp || par_overlap <= snap_par {
            continue;
        }

        let area =
            (r.bbox.2 - r.bbox.0).into_inner().abs() * (r.bbox.3 - r.bbox.1).into_inner().abs();
        if area > best_area {
            best_area = area;
            best_color = Some((
                r.fill_color.red(),
                r.fill_color.green(),
                r.fill_color.blue(),
            ));
        }
    }

    best_color
}

/// Returns the RGB fill color of the largest qualifying rect that **fully contains**
/// `edge` in the perpendicular dimension with overlap > `snap_par_tol` in the
/// parallel dimension.
///
/// This detects the case where a thin artifact edge is embedded inside a large filled
/// rect rather than sitting at its boundary (and would therefore have zero adjacent rects).
fn find_containing_rect_color(
    rects: &[Rect],
    edge: &Edge,
    snap_x_tol: f32,
    snap_y_tol: f32,
) -> Option<(u8, u8, u8)> {
    let mut best_area = 0.0f32;
    let mut best_color: Option<(u8, u8, u8)> = None;

    for r in rects {
        if !is_qualifying_rect(r, snap_x_tol, snap_y_tol) {
            continue;
        }

        let (rx0, ry0, rx1, ry1) = (
            r.bbox.0.into_inner(),
            r.bbox.1.into_inner(),
            r.bbox.2.into_inner(),
            r.bbox.3.into_inner(),
        );

        let contained = match edge.orientation {
            Orientation::Horizontal => {
                let y = edge.y1.into_inner();
                let x_overlap = rx1.min(edge.x2.into_inner()) - rx0.max(edge.x1.into_inner());
                // Edge's y must be strictly inside the rect's y range
                y > ry0 && y < ry1 && x_overlap > snap_x_tol
            }
            Orientation::Vertical => {
                let x = edge.x1.into_inner();
                let y_overlap = ry1.min(edge.y2.into_inner()) - ry0.max(edge.y1.into_inner());
                x > rx0 && x < rx1 && y_overlap > snap_y_tol
            }
        };

        if !contained {
            continue;
        }

        let area = (rx1 - rx0) * (ry1 - ry0);
        if area > best_area {
            best_area = area;
            best_color = Some((
                r.fill_color.red(),
                r.fill_color.green(),
                r.fill_color.blue(),
            ));
        }
    }

    best_color
}

/// Filters out edges that are invisible against the page background by checking the
/// fill colors of immediately adjacent and containing rectangles.
///
/// An edge is **excluded** when it is indistinguishable from its surroundings:
///
/// - **Two adjacent rects found** (one on each side): exclude if *both* have the same
///   fill color as the edge.  If at least one side has a different color the edge is
///   visible and is kept.
/// - **One adjacent rect found**: treat the missing side as the default PDF page
///   background (white).  Exclude only when the edge color matches *both* the adjacent
///   rect *and* white — i.e. exclude only white-on-white.  Any non-white edge is
///   visible from the page-white side and is kept.
/// - **Zero adjacent rects – containing rect found**: exclude if the containing rect's
///   fill color matches the edge color (artifact embedded in a same-colored region).
/// - **Zero adjacent rects – no containing rect**: exclude only if the edge is white
///   (`255, 255, 255`), the standard invisible-on-default-PDF-background case.
fn filter_edges_invisible_against_background(
    edges: &mut Vec<Edge>,
    rects: &[Rect],
    snap_x_tol: f32,
    snap_y_tol: f32,
) {
    const WHITE: (u8, u8, u8) = (255, 255, 255);

    edges.retain(|edge| {
        let color = (edge.color.red(), edge.color.green(), edge.color.blue());

        let side_a = find_adjacent_rect_color(rects, edge, true, snap_x_tol, snap_y_tol);
        let side_b = find_adjacent_rect_color(rects, edge, false, snap_x_tol, snap_y_tol);

        match (side_a, side_b) {
            // Both sides have explicit adjacent rects.
            (Some(ca), Some(cb)) => ca != color || cb != color,

            // Exactly one side has an adjacent rect.  Treat the missing side as the
            // default PDF page background (white).  An edge is invisible — and thus
            // excluded — only when it is the same color as BOTH the adjacent rect
            // AND the default white background, i.e. only when the edge is white and
            // the adjacent rect is also white.  Any non-white edge is visible from the
            // page-white side and must be kept.
            (Some(ca), None) | (None, Some(ca)) => ca != color || color != WHITE,

            // No adjacent rects at all.  Check for a containing rect first (handles
            // artifacts embedded inside a filled region), then fall back to the
            // page-white rule.
            (None, None) => match find_containing_rect_color(rects, edge, snap_x_tol, snap_y_tol) {
                Some(c) => c != color,
                None => color != WHITE,
            },
        }
    });
}

/// Clips edges to a bounding box region.
///
/// Edges that intersect with the clip region are clipped to fit within it.
/// Edges completely outside the clip region are removed.
///
/// # Arguments
///
/// * `edges` - The edges to clip (modified in place).
/// * `clip` - The clip region as (x1, y1, x2, y2).
fn clip_edges_to_bbox(edges: &mut HashMap<Orientation, Vec<Edge>>, clip: &BboxKey) {
    let (clip_x1, clip_y1, clip_x2, clip_y2) = *clip;

    // Clip horizontal edges
    if let Some(h_edges) = edges.get_mut(&Orientation::Horizontal) {
        h_edges.retain_mut(|edge| {
            // For horizontal edges: y1 == y2
            // Check if the edge's y coordinate is within clip region
            if edge.y1 < clip_y1 || edge.y1 > clip_y2 {
                return false;
            }
            // Check if edge intersects with clip region horizontally
            if edge.x2 <= clip_x1 || edge.x1 >= clip_x2 {
                return false;
            }
            // Clip the edge to the clip region
            if edge.x1 < clip_x1 {
                edge.x1 = clip_x1;
            }
            if edge.x2 > clip_x2 {
                edge.x2 = clip_x2;
            }
            true
        });
    }

    // Clip vertical edges
    if let Some(v_edges) = edges.get_mut(&Orientation::Vertical) {
        v_edges.retain_mut(|edge| {
            // For vertical edges: x1 == x2
            // Check if the edge's x coordinate is within clip region
            if edge.x1 < clip_x1 || edge.x1 > clip_x2 {
                return false;
            }
            // Check if edge intersects with clip region vertically
            if edge.y2 <= clip_y1 || edge.y1 >= clip_y2 {
                return false;
            }
            // Clip the edge to the clip region
            if edge.y1 < clip_y1 {
                edge.y1 = clip_y1;
            }
            if edge.y2 > clip_y2 {
                edge.y2 = clip_y2;
            }
            true
        });
    }
}

/// If one strat is Text and the other is not Text, we need to extend Text edges to its neighbor,
/// because Text edges are usually too short to intersect with edges vertical to itself.
/// Please see #8(https://github.com/monchin/tablers/issues/8) for more details.
///
/// # Arguments
///
/// * `edges_to_extend` - The edges to extend.
/// * `edges_the_other_orientation` - The edges of the other orientation.
/// * `extend_orientation` - The orientation of the edges to extend.
/// * `intersection_tolerance` - The intersection tolerance.
fn extend_edges_to_neighbors(
    edges_to_extend: &mut [Edge],
    edges_the_other_orientation: &[Edge],
    extend_orientation: Orientation,
    intersection_tolerance: OrderedFloat<f32>,
) {
    let (first_key_to_extend, second_key_to_extend): (EdgePropGetter, EdgePropGetter) =
        match extend_orientation {
            Orientation::Horizontal => (|e| e.x1, |e| e.x2),
            Orientation::Vertical => (|e| e.y1, |e| e.y2),
        };
    let location_key: EdgePropGetter = match extend_orientation {
        Orientation::Horizontal => |e| e.y1,
        Orientation::Vertical => |e| e.x1,
    };
    let the_other_orientation_key: EdgePropGetter = match extend_orientation {
        Orientation::Horizontal => |e| e.x1,
        Orientation::Vertical => |e| e.y1,
    };

    let (first_key_range, second_key_range): (EdgePropGetter, EdgePropGetter) =
        match extend_orientation {
            Orientation::Horizontal => (|e| e.y1, |e| e.y2),
            Orientation::Vertical => (|e| e.x1, |e| e.x2),
        };

    let (set_first_val, set_second_val): (EdgePropSetter, EdgePropSetter) = match extend_orientation
    {
        Orientation::Horizontal => (|e, v| e.x1 = v, |e, v| e.x2 = v),
        Orientation::Vertical => (|e, v| e.y1 = v, |e, v| e.y2 = v),
    };

    for edge_to_extend in edges_to_extend.iter_mut() {
        let loc = location_key(edge_to_extend);
        let (first_val_to_extend, second_val_to_extend) = (
            first_key_to_extend(edge_to_extend),
            second_key_to_extend(edge_to_extend),
        );
        // Use indices instead of cloning edges to improve performance
        let mut intersecting_edge_indices: Vec<usize> = edges_the_other_orientation
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                let (range1, range2) = (first_key_range(e), second_key_range(e));
                loc - range2 <= intersection_tolerance && range1 - loc <= intersection_tolerance
            })
            .map(|(i, _)| i)
            .collect();
        let n_intersecting_edges = intersecting_edge_indices.len();
        if n_intersecting_edges > 1 {
            // Sort indices by the_other_orientation_key to ensure correct ordering
            intersecting_edge_indices.sort_by(|&i, &j| {
                let key_i = the_other_orientation_key(&edges_the_other_orientation[i]);
                let key_j = the_other_orientation_key(&edges_the_other_orientation[j]);
                key_i
                    .partial_cmp(&key_j)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            // Extend first value (left for horizontal, top for vertical)
            for i in 0..n_intersecting_edges {
                let idx = intersecting_edge_indices[i];
                let loc_the_other_orientation =
                    the_other_orientation_key(&edges_the_other_orientation[idx]);
                if (first_val_to_extend - loc_the_other_orientation) < -intersection_tolerance {
                    if i != 0 {
                        let prev_idx = intersecting_edge_indices[i - 1];
                        set_first_val(
                            edge_to_extend,
                            the_other_orientation_key(&edges_the_other_orientation[prev_idx]),
                        );
                    }
                    break;
                }
            }

            // Extend second value (right for horizontal, bottom for vertical)
            for i in (0..n_intersecting_edges).rev() {
                let idx = intersecting_edge_indices[i];
                let loc_the_other_orientation =
                    the_other_orientation_key(&edges_the_other_orientation[idx]);
                if (second_val_to_extend - loc_the_other_orientation) > -intersection_tolerance {
                    if i != n_intersecting_edges - 1 {
                        let next_idx = intersecting_edge_indices[i + 1];
                        set_second_val(
                            edge_to_extend,
                            the_other_orientation_key(&edges_the_other_orientation[next_idx]),
                        );
                    }
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod extend_edges_to_neighbors_tests {
    use super::extend_edges_to_neighbors;
    use crate::edges::Edge;
    use crate::objects::Orientation;
    use ordered_float::OrderedFloat;
    use pdfium_render::prelude::PdfColor;

    /// Creates a test edge
    fn create_edge(x1: f32, y1: f32, x2: f32, y2: f32, orientation: Orientation) -> Edge {
        Edge {
            x1: OrderedFloat(x1),
            y1: OrderedFloat(y1),
            x2: OrderedFloat(x2),
            y2: OrderedFloat(y2),
            orientation,
            width: OrderedFloat(1.0),
            color: PdfColor::new(0, 0, 0, 255),
        }
    }

    /// Test extending horizontal edges to neighbor edges
    #[test]
    fn test_extend_horizontal_edges_to_neighbors() {
        // Create horizontal edge to extend (y=5, x from 2 to 3)
        let mut edges_to_extend = vec![create_edge(2.0, 5.0, 3.0, 5.0, Orientation::Horizontal)];

        // Create vertical edges as references (x=1, y from 0 to 10; x=4, y from 0 to 10; x=6, y from 0 to 10)
        let edges_the_other_orientation = vec![
            create_edge(1.0, 0.0, 1.0, 10.0, Orientation::Vertical),
            create_edge(4.0, 0.0, 4.0, 10.0, Orientation::Vertical),
            create_edge(6.0, 0.0, 6.0, 10.0, Orientation::Vertical),
        ];

        let intersection_tolerance = OrderedFloat(0.5);
        extend_edges_to_neighbors(
            &mut edges_to_extend,
            &edges_the_other_orientation,
            Orientation::Horizontal,
            intersection_tolerance,
        );

        // Check if edge is correctly extended to neighbor vertical edges
        let extended_edge = &edges_to_extend[0];
        assert_eq!(extended_edge.x1, OrderedFloat(1.0)); // Should extend to first vertical edge
        assert_eq!(extended_edge.x2, OrderedFloat(4.0)); // Should extend to second vertical edge
        assert_eq!(extended_edge.y1, OrderedFloat(5.0)); // y coordinate should remain unchanged
        assert_eq!(extended_edge.y2, OrderedFloat(5.0)); // y coordinate should remain unchanged
    }

    /// Test extending vertical edges to neighbor edges
    #[test]
    fn test_extend_vertical_edges_to_neighbors() {
        // Create vertical edge to extend (x=5, y from 2 to 3)
        let mut edges_to_extend = vec![create_edge(5.0, 2.0, 5.0, 3.0, Orientation::Vertical)];

        // Create horizontal edges as references (y=1, x from 0 to 10; y=4, x from 0 to 10; y=6, x from 0 to 10)
        let edges_the_other_orientation = vec![
            create_edge(0.0, 1.0, 10.0, 1.0, Orientation::Horizontal),
            create_edge(0.0, 4.0, 10.0, 4.0, Orientation::Horizontal),
            create_edge(0.0, 6.0, 10.0, 6.0, Orientation::Horizontal),
        ];

        let intersection_tolerance = OrderedFloat(0.5);
        extend_edges_to_neighbors(
            &mut edges_to_extend,
            &edges_the_other_orientation,
            Orientation::Vertical,
            intersection_tolerance,
        );

        // Check if edge is correctly extended to neighbor horizontal edges
        let extended_edge = &edges_to_extend[0];
        assert_eq!(extended_edge.y1, OrderedFloat(1.0)); // Should extend to first horizontal edge
        assert_eq!(extended_edge.y2, OrderedFloat(4.0)); // Should extend to second horizontal edge
        assert_eq!(extended_edge.x1, OrderedFloat(5.0)); // x coordinate should remain unchanged
        assert_eq!(extended_edge.x2, OrderedFloat(5.0)); // x coordinate should remain unchanged
    }

    /// Test case where edges don't need extension (not enough neighbor edges)
    #[test]
    fn test_no_extension_with_few_edges() {
        // Create horizontal edge to extend
        let mut edges_to_extend = vec![create_edge(2.0, 5.0, 3.0, 5.0, Orientation::Horizontal)];

        // Only one vertical edge, not enough for extension
        let edges_the_other_orientation =
            vec![create_edge(1.0, 0.0, 1.0, 10.0, Orientation::Vertical)];

        let original_edge = edges_to_extend[0].clone();
        let intersection_tolerance = OrderedFloat(0.5);
        extend_edges_to_neighbors(
            &mut edges_to_extend,
            &edges_the_other_orientation,
            Orientation::Horizontal,
            intersection_tolerance,
        );

        // Edge should remain unchanged
        assert_eq!(edges_to_extend[0].x1, original_edge.x1);
        assert_eq!(edges_to_extend[0].x2, original_edge.x2);
    }

    /// Test case where edge is shorter than neighbor spacing (should extend)
    #[test]
    fn test_edge_shorter_than_neighbor_spacing() {
        // Create horizontal edge that doesn't reach all vertical edges (y=5, x from 2 to 5)
        let mut edges_to_extend = vec![create_edge(2.0, 5.0, 5.0, 5.0, Orientation::Horizontal)];

        // Create vertical edges (x=1, y from 0 to 10; x=4, y from 0 to 10; x=6, y from 0 to 10)
        let edges_the_other_orientation = vec![
            create_edge(1.0, 0.0, 1.0, 10.0, Orientation::Vertical),
            create_edge(4.0, 0.0, 4.0, 10.0, Orientation::Vertical),
            create_edge(6.0, 0.0, 6.0, 10.0, Orientation::Vertical),
        ];

        let intersection_tolerance = OrderedFloat(0.5);
        extend_edges_to_neighbors(
            &mut edges_to_extend,
            &edges_the_other_orientation,
            Orientation::Horizontal,
            intersection_tolerance,
        );

        // Edge should extend to reach the outer vertical edges
        assert_eq!(edges_to_extend[0].x1, OrderedFloat(1.0));
        assert_eq!(edges_to_extend[0].x2, OrderedFloat(6.0));
    }

    /// Test extending multiple edges simultaneously
    #[test]
    fn test_multiple_edges_extension() {
        // Create multiple horizontal edges to extend
        let mut edges_to_extend = vec![
            create_edge(2.0, 3.0, 3.0, 3.0, Orientation::Horizontal), // y=3, x from 2 to 3
            create_edge(2.0, 7.0, 3.0, 7.0, Orientation::Horizontal), // y=7, x from 2 to 3
        ];

        // Create vertical edges
        let edges_the_other_orientation = vec![
            create_edge(1.0, 0.0, 1.0, 10.0, Orientation::Vertical),
            create_edge(4.0, 0.0, 4.0, 10.0, Orientation::Vertical),
            create_edge(6.0, 0.0, 6.0, 10.0, Orientation::Vertical),
        ];

        let intersection_tolerance = OrderedFloat(0.5);
        extend_edges_to_neighbors(
            &mut edges_to_extend,
            &edges_the_other_orientation,
            Orientation::Horizontal,
            intersection_tolerance,
        );

        // Check both edges are correctly extended
        assert_eq!(edges_to_extend[0].x1, OrderedFloat(1.0));
        assert_eq!(edges_to_extend[0].x2, OrderedFloat(4.0));
        assert_eq!(edges_to_extend[0].y1, OrderedFloat(3.0));

        assert_eq!(edges_to_extend[1].x1, OrderedFloat(1.0));
        assert_eq!(edges_to_extend[1].x2, OrderedFloat(4.0));
        assert_eq!(edges_to_extend[1].y1, OrderedFloat(7.0));
    }

    /// Test case where edge is already fully covered by intersecting edges
    #[test]
    fn test_edge_already_fully_covered() {
        // Create horizontal edge that already spans the full range (y=5, x from 1 to 6)
        let mut edges_to_extend = vec![create_edge(1.0, 5.0, 6.0, 5.0, Orientation::Horizontal)];

        // Create vertical edges that the horizontal edge already spans
        let edges_the_other_orientation = vec![
            create_edge(1.0, 0.0, 1.0, 10.0, Orientation::Vertical),
            create_edge(4.0, 0.0, 4.0, 10.0, Orientation::Vertical),
            create_edge(6.0, 0.0, 6.0, 10.0, Orientation::Vertical),
        ];

        let original_edge = edges_to_extend[0].clone();
        let intersection_tolerance = OrderedFloat(0.5);
        extend_edges_to_neighbors(
            &mut edges_to_extend,
            &edges_the_other_orientation,
            Orientation::Horizontal,
            intersection_tolerance,
        );

        // Edge should remain unchanged since it already covers the range
        assert_eq!(edges_to_extend[0].x1, original_edge.x1);
        assert_eq!(edges_to_extend[0].x2, original_edge.x2);
    }

    /// Test case with overlapping edges in the other orientation
    #[test]
    fn test_overlapping_edges_in_other_orientation() {
        // Create horizontal edge to extend (y=5, x from 2 to 3)
        let mut edges_to_extend = vec![create_edge(2.0, 5.0, 3.0, 5.0, Orientation::Horizontal)];

        // Create overlapping vertical edges (some have overlapping x positions)
        let edges_the_other_orientation = vec![
            create_edge(1.0, 0.0, 1.0, 10.0, Orientation::Vertical),
            create_edge(1.5, 0.0, 1.5, 10.0, Orientation::Vertical), // Overlaps with x=1
            create_edge(4.0, 0.0, 4.0, 10.0, Orientation::Vertical),
            create_edge(4.2, 0.0, 4.2, 10.0, Orientation::Vertical), // Overlaps with x=4
            create_edge(6.0, 0.0, 6.0, 10.0, Orientation::Vertical),
        ];

        let intersection_tolerance = OrderedFloat(0.5);
        extend_edges_to_neighbors(
            &mut edges_to_extend,
            &edges_the_other_orientation,
            Orientation::Horizontal,
            intersection_tolerance,
        );

        // Algorithm extends to the nearest neighbor edges:
        // For x1=2.0, it finds x=4.0 is beyond (2.0 - 4.0 = -2.0 < -0.5), so uses the previous edge x=1.5
        // For x2=3.0, it finds x=1.5 is before (3.0 - 1.5 = 1.5 > -0.5), so uses the next edge x=4.0
        let extended_edge = &edges_to_extend[0];
        assert_eq!(extended_edge.x1, OrderedFloat(1.5)); // Extends to nearest neighbor before x=4.0
        assert_eq!(extended_edge.x2, OrderedFloat(4.0)); // Extends to nearest neighbor after x=1.5
    }

    /// Test case where edge is completely outside the range of intersecting edges
    #[test]
    fn test_edge_outside_intersecting_range() {
        // Create horizontal edge that doesn't intersect with any vertical edges (y=15, x from 2 to 3)
        let mut edges_to_extend = vec![create_edge(2.0, 15.0, 3.0, 15.0, Orientation::Horizontal)];

        // Create vertical edges that don't intersect with the horizontal edge
        let edges_the_other_orientation = vec![
            create_edge(1.0, 0.0, 1.0, 10.0, Orientation::Vertical), // y range: 0-10
            create_edge(4.0, 0.0, 4.0, 10.0, Orientation::Vertical), // y range: 0-10
            create_edge(6.0, 0.0, 6.0, 10.0, Orientation::Vertical), // y range: 0-10
        ];

        let original_edge = edges_to_extend[0].clone();
        let intersection_tolerance = OrderedFloat(0.5);
        extend_edges_to_neighbors(
            &mut edges_to_extend,
            &edges_the_other_orientation,
            Orientation::Horizontal,
            intersection_tolerance,
        );

        // Edge should remain unchanged since no edges intersect
        assert_eq!(edges_to_extend[0].x1, original_edge.x1);
        assert_eq!(edges_to_extend[0].x2, original_edge.x2);
        assert_eq!(edges_to_extend[0].y1, original_edge.y1);
        assert_eq!(edges_to_extend[0].y2, original_edge.y2);
    }

    /// Test case where edge extends beyond the first/last intersecting edge
    #[test]
    fn test_edge_extends_beyond_boundaries() {
        // Create horizontal edge that extends beyond the vertical edges (y=5, x from 0 to 7)
        let mut edges_to_extend = vec![create_edge(0.0, 5.0, 7.0, 5.0, Orientation::Horizontal)];

        // Create vertical edges in the middle
        let edges_the_other_orientation = vec![
            create_edge(2.0, 0.0, 2.0, 10.0, Orientation::Vertical),
            create_edge(4.0, 0.0, 4.0, 10.0, Orientation::Vertical),
            create_edge(5.0, 0.0, 5.0, 10.0, Orientation::Vertical),
        ];

        let intersection_tolerance = OrderedFloat(0.5);
        extend_edges_to_neighbors(
            &mut edges_to_extend,
            &edges_the_other_orientation,
            Orientation::Horizontal,
            intersection_tolerance,
        );

        // Edge should remain unchanged since it already extends beyond all intersecting edges
        let extended_edge = &edges_to_extend[0];
        assert_eq!(extended_edge.x1, OrderedFloat(0.0));
        assert_eq!(extended_edge.x2, OrderedFloat(7.0));
    }

    /// Test case with unsorted edges in the other orientation
    #[test]
    fn test_unsorted_edges_handling() {
        // Create horizontal edge to extend (y=5, x from 2 to 3)
        let mut edges_to_extend = vec![create_edge(2.0, 5.0, 3.0, 5.0, Orientation::Horizontal)];

        // Create vertical edges in unsorted order
        let edges_the_other_orientation = vec![
            create_edge(6.0, 0.0, 6.0, 10.0, Orientation::Vertical), // x=6 (last)
            create_edge(1.0, 0.0, 1.0, 10.0, Orientation::Vertical), // x=1 (first)
            create_edge(4.0, 0.0, 4.0, 10.0, Orientation::Vertical), // x=4 (middle)
        ];

        let intersection_tolerance = OrderedFloat(0.5);
        extend_edges_to_neighbors(
            &mut edges_to_extend,
            &edges_the_other_orientation,
            Orientation::Horizontal,
            intersection_tolerance,
        );

        // Should correctly extend to first and last edges despite unsorted input
        let extended_edge = &edges_to_extend[0];
        assert_eq!(extended_edge.x1, OrderedFloat(1.0)); // Should extend to first vertical edge (x=1)
        assert_eq!(extended_edge.x2, OrderedFloat(4.0)); // Should extend to middle vertical edge (x=4)
    }
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
                    .push((*h).clone());
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
/// Groups cells into tables based on shared corners.
///
/// This function only groups cells - it does not perform any filtering.
/// All filtering (single cell, min_rows, min_columns) should be done after this function.
pub fn cells_to_tables(cells: &[BboxKey]) -> Vec<Vec<BboxKey>> {
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

    tables
}

/// Counts the number of rows in a table (based on unique y1 values).
fn count_rows(cells: &[BboxKey]) -> usize {
    cells
        .iter()
        .map(|c| OrderedFloat(c.1))
        .collect::<std::collections::BTreeSet<_>>()
        .len()
}

/// Counts the number of columns in a table (based on unique x1 values).
fn count_cols(cells: &[BboxKey]) -> usize {
    cells
        .iter()
        .map(|c| OrderedFloat(c.0))
        .collect::<std::collections::BTreeSet<_>>()
        .len()
}

/// Filters tables based on settings criteria.
fn filter_tables(
    tables: Vec<Vec<BboxKey>>,
    include_single_cell: bool,
    min_rows: Option<usize>,
    min_columns: Option<usize>,
) -> Vec<Vec<BboxKey>> {
    let tables_after_filter_single_cell: Vec<Vec<BboxKey>> = match include_single_cell {
        true => tables,
        false => tables.into_iter().filter(|t| t.len() > 1).collect(),
    };
    let tables_after_filter_rows = match min_rows {
        Some(min_r) => tables_after_filter_single_cell
            .into_iter()
            .filter(|t| count_rows(t) >= min_r)
            .collect(),
        None => tables_after_filter_single_cell,
    };
    match min_columns {
        Some(min_c) => tables_after_filter_rows
            .into_iter()
            .filter(|t| count_cols(t) >= min_c)
            .collect(),
        None => tables_after_filter_rows,
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

    /// Extracts and processes edges from a PDF page or explicit edges.
    ///
    /// # Arguments
    ///
    /// * `page` - The PDF page to extract edges from. Can be None only if both
    ///   horizontal_strategy and vertical_strategy are set to Explicit.
    ///
    /// # Returns
    ///
    /// A HashMap of edges grouped by orientation.
    ///
    /// # Panics
    ///
    /// Panics if page is None and either strategy is not Explicit.
    pub(crate) fn get_edges(&self, page: Option<&Page>) -> HashMap<Orientation, Vec<Edge>> {
        let settings = self.settings.as_ref();

        let edges_all = if let Some(page) = page {
            let objects_opt = page.objects.borrow();
            if objects_opt.is_none() {
                page.extract_objects();
            }
            let objects = objects_opt.as_ref().expect("Objects should be extracted");
            make_edges(objects, self.settings.clone())
        } else {
            // Page is None, verify both strategies are Explicit
            if settings.horizontal_strategy != StrategyType::Explicit
                || settings.vertical_strategy != StrategyType::Explicit
            {
                panic!(
                    "Page can only be None when both horizontal_strategy and vertical_strategy are 'explicit'"
                );
            }
            // Create edges only from explicit edges
            let mut edges = HashMap::new();
            edges.insert(
                Orientation::Horizontal,
                settings.explicit_h_edges.clone().unwrap_or_default(),
            );
            edges.insert(
                Orientation::Vertical,
                settings.explicit_v_edges.clone().unwrap_or_default(),
            );
            edges
        };

        let mut v_edges = edges_all
            .get(&Orientation::Vertical)
            .cloned()
            .unwrap_or_default();
        let mut h_edges = edges_all
            .get(&Orientation::Horizontal)
            .cloned()
            .unwrap_or_default();

        if settings.exclude_background_colored_edges {
            let snap_x = settings.snap_x_tolerance.into_inner();
            let snap_y = settings.snap_y_tolerance.into_inner();
            let rects: Vec<Rect> = if let Some(page) = page {
                let objects_opt = page.objects.borrow();
                objects_opt
                    .as_ref()
                    .map_or_else(Vec::new, |objects| objects.rects.clone())
            } else {
                Vec::new()
            };
            filter_edges_invisible_against_background(&mut v_edges, &rects, snap_x, snap_y);
            filter_edges_invisible_against_background(&mut h_edges, &rects, snap_x, snap_y);
        }

        filter_edges_by_min_len(&mut v_edges, *settings.edge_min_length_prefilter);
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
            .get_mut(&Orientation::Vertical)
            .unwrap()
            .sort_by_key(|e| (e.x1, e.y1));
        edges_merged
            .get_mut(&Orientation::Horizontal)
            .unwrap()
            .sort_by_key(|e| (e.y1, e.x1));
        edges_merged
    }

    /// Computes intersection points from a set of horizontal and vertical edges.
    ///
    /// # Arguments
    ///
    /// * `h_edges` - A vector of horizontal edges.
    /// * `v_edges` - A vector of vertical edges.
    ///
    /// # Returns
    ///
    /// A HashMap mapping each intersection point `(x, y)` to a map of
    /// `Orientation -> Vec<Edge>` indicating which edges pass through that point.
    pub(crate) fn get_intersections_from_edges(
        &self,
        h_edges: Vec<Edge>,
        v_edges: Vec<Edge>,
    ) -> HashMap<Point, HashMap<Orientation, Vec<Edge>>> {
        let mut edges = HashMap::new();
        edges.insert(Orientation::Horizontal, h_edges);
        edges.insert(Orientation::Vertical, v_edges);
        edges_to_intersections(
            &mut edges,
            *self.settings.intersection_x_tolerance,
            *self.settings.intersection_y_tolerance,
        )
    }
}

/// Detects missing boundary cells caused by unclosed table edges and returns them.
///
/// For each of the four sides of the detected table, the function checks whether
/// *every* outermost intersection point has a corresponding edge that extends past
/// the table boundary by more than the given tolerance.  If so, a virtual closing
/// edge is synthesised at the outermost extension endpoint and the missing boundary
/// cells are returned.
///
/// Inner cells from the first-pass detection are never recomputed – only the new
/// boundary cells (at most one extra column/row per side) are returned.
///
/// All four checks are **skipped entirely** when either `h_strategy` or
/// `v_strategy` is `Text`.  In mixed-strategy configurations (one `Text`, one
/// `Lines`), text-derived edges can extend across table boundaries in ways that
/// produce false-positive missing columns or rows on any of the four sides.
/// The checks are only reliable when both strategies produce real PDF lines.
///
/// # Arguments
///
/// * `table_cells` - Bounding boxes of cells already detected for this table.
/// * `h_edges`     - All horizontal edges on the page (post-merge).
/// * `v_edges`     - All vertical edges on the page (post-merge).
/// * `x_tol`       - X-axis tolerance (reuses `intersection_x_tolerance`).
/// * `y_tol`       - Y-axis tolerance (reuses `intersection_y_tolerance`).
/// * `h_strategy`  - Active horizontal strategy.
/// * `v_strategy`  - Active vertical strategy.
///
/// # Returns
///
/// A (possibly empty) vector of new cell bounding boxes to append.
fn collect_unclosed_boundary_cells(
    table_cells: &[BboxKey],
    h_edges: &[Edge],
    v_edges: &[Edge],
    x_tol: f32,
    y_tol: f32,
    h_strategy: StrategyType,
    v_strategy: StrategyType,
) -> Vec<BboxKey> {
    if table_cells.is_empty() {
        return Vec::new();
    }

    let x_tol = OrderedFloat(x_tol);
    let y_tol = OrderedFloat(y_tol);

    let min_x = table_cells.iter().map(|c| c.0).min().unwrap();
    let max_x = table_cells.iter().map(|c| c.2).max().unwrap();
    let min_y = table_cells.iter().map(|c| c.1).min().unwrap();
    let max_y = table_cells.iter().map(|c| c.3).max().unwrap();

    // When either strategy is Text, the corresponding edges are derived from text
    // positions rather than real PDF lines.  Text-derived edges can extend across
    // table boundaries in ways that trigger false-positive missing columns/rows on
    // every side, so we skip all four checks in that case.
    if h_strategy == StrategyType::Text || v_strategy == StrategyType::Text {
        return Vec::new();
    }

    let mut new_cells: Vec<BboxKey> = Vec::new();

    // ── Left side ──────────────────────────────────────────────────────────────
    // Collect all y-coordinates that appear as corners of left-boundary cells.
    // For each such y, look for a horizontal edge that crosses the left boundary
    // and continues further left (h.x1 < min_x - x_tol).
    {
        let boundary_ys: BTreeSet<OrderedFloat<f32>> = table_cells
            .iter()
            .filter(|c| c.0 == min_x)
            .flat_map(|c| [c.1, c.3])
            .collect();

        if !boundary_ys.is_empty() {
            // For every boundary y, find the rightmost (closest) left extension.
            // Using max() ensures that all h-edges in the new column reach new_x.
            let extending: Vec<OrderedFloat<f32>> = boundary_ys
                .iter()
                .filter_map(|&y| {
                    h_edges
                        .iter()
                        .filter(|h| {
                            h.y1 >= y - y_tol
                                && h.y1 <= y + y_tol
                                && h.x1 < min_x - x_tol
                                && h.x2 >= min_x - x_tol
                        })
                        .map(|h| h.x1)
                        .min()
                })
                .collect();

            if extending.len() == boundary_ys.len() {
                let new_x = extending.iter().cloned().max().unwrap();
                let ys: Vec<OrderedFloat<f32>> = boundary_ys.into_iter().collect();
                for i in 0..ys.len() - 1 {
                    new_cells.push((new_x, ys[i], min_x, ys[i + 1]));
                }
            }
        }
    }

    // ── Right side ─────────────────────────────────────────────────────────────
    {
        let boundary_ys: BTreeSet<OrderedFloat<f32>> = table_cells
            .iter()
            .filter(|c| c.2 == max_x)
            .flat_map(|c| [c.1, c.3])
            .collect();

        if !boundary_ys.is_empty() {
            let extending: Vec<OrderedFloat<f32>> = boundary_ys
                .iter()
                .filter_map(|&y| {
                    h_edges
                        .iter()
                        .filter(|h| {
                            h.y1 >= y - y_tol
                                && h.y1 <= y + y_tol
                                && h.x2 > max_x + x_tol
                                && h.x1 <= max_x + x_tol
                        })
                        .map(|h| h.x2)
                        .max()
                })
                .collect();

            if extending.len() == boundary_ys.len() {
                let new_x = extending.iter().cloned().min().unwrap();
                let ys: Vec<OrderedFloat<f32>> = boundary_ys.into_iter().collect();
                for i in 0..ys.len() - 1 {
                    new_cells.push((max_x, ys[i], new_x, ys[i + 1]));
                }
            }
        }
    }

    // ── Top side ───────────────────────────────────────────────────────────────
    {
        let boundary_xs: BTreeSet<OrderedFloat<f32>> = table_cells
            .iter()
            .filter(|c| c.1 == min_y)
            .flat_map(|c| [c.0, c.2])
            .collect();

        if !boundary_xs.is_empty() {
            let extending: Vec<OrderedFloat<f32>> = boundary_xs
                .iter()
                .filter_map(|&x| {
                    v_edges
                        .iter()
                        .filter(|v| {
                            v.x1 >= x - x_tol
                                && v.x1 <= x + x_tol
                                && v.y1 < min_y - y_tol
                                && v.y2 >= min_y - y_tol
                        })
                        .map(|v| v.y1)
                        .min()
                })
                .collect();

            if extending.len() == boundary_xs.len() {
                let new_y = extending.iter().cloned().max().unwrap();
                let xs: Vec<OrderedFloat<f32>> = boundary_xs.into_iter().collect();
                for i in 0..xs.len() - 1 {
                    new_cells.push((xs[i], new_y, xs[i + 1], min_y));
                }
            }
        }
    }

    // ── Bottom side ────────────────────────────────────────────────────────────
    {
        let boundary_xs: BTreeSet<OrderedFloat<f32>> = table_cells
            .iter()
            .filter(|c| c.3 == max_y)
            .flat_map(|c| [c.0, c.2])
            .collect();

        if !boundary_xs.is_empty() {
            let extending: Vec<OrderedFloat<f32>> = boundary_xs
                .iter()
                .filter_map(|&x| {
                    v_edges
                        .iter()
                        .filter(|v| {
                            v.x1 >= x - x_tol
                                && v.x1 <= x + x_tol
                                && v.y2 > max_y + y_tol
                                && v.y1 <= max_y + y_tol
                        })
                        .map(|v| v.y2)
                        .max()
                })
                .collect();

            if extending.len() == boundary_xs.len() {
                let new_y = extending.iter().cloned().min().unwrap();
                let xs: Vec<OrderedFloat<f32>> = boundary_xs.into_iter().collect();
                for i in 0..xs.len() - 1 {
                    new_cells.push((xs[i], max_y, xs[i + 1], new_y));
                }
            }
        }
    }

    new_cells
}

/// Finds all table cell bounding boxes in a PDF page or from explicit edges.
///
/// # Arguments
///
/// * `pdf_page` - The PDF page to analyze. Can be None only if both
///   horizontal_strategy and vertical_strategy are set to Explicit.
/// * `tf_settings` - The table finder settings.
/// * `clip` - Optional clip region. If provided, only edges within this region
///   are used for cell detection. Edges intersecting the clip boundary are
///   clipped to fit within it.
///
/// # Returns
///
/// A vector of bounding boxes for detected cells.
///
/// # Panics
///
/// Panics if pdf_page is None and either strategy is not Explicit.
pub fn find_all_cells_bboxes(
    pdf_page: Option<&Page>,
    tf_settings: Rc<TfSettings>,
    clip: Option<&BboxKey>,
) -> Vec<BboxKey> {
    let table_finder = TableFinder::new(tf_settings.clone());
    let mut edges = table_finder.get_edges(pdf_page);

    // Apply clip if provided
    if let Some(clip_bbox) = clip {
        clip_edges_to_bbox(&mut edges, clip_bbox);
    }

    let (h_strat, v_strat) = (
        tf_settings.horizontal_strategy,
        tf_settings.vertical_strategy,
    );
    if h_strat == StrategyType::Text && v_strat != StrategyType::Text {
        let v_edges = edges.get(&Orientation::Vertical).unwrap().clone();
        extend_edges_to_neighbors(
            edges.get_mut(&Orientation::Horizontal).unwrap(),
            &v_edges,
            Orientation::Horizontal,
            tf_settings.intersection_x_tolerance.into(),
        );
    } else if v_strat == StrategyType::Text && h_strat != StrategyType::Text {
        let h_edges = edges.get(&Orientation::Horizontal).unwrap().clone();
        extend_edges_to_neighbors(
            edges.get_mut(&Orientation::Vertical).unwrap(),
            &h_edges,
            Orientation::Vertical,
            tf_settings.intersection_y_tolerance.into(),
        );
    }

    let intersections = edges_to_intersections(
        &mut edges.clone(),
        *table_finder.settings.intersection_x_tolerance,
        *table_finder.settings.intersection_y_tolerance,
    );
    let mut cells = intersections_to_cells(intersections);

    if tf_settings.close_unclosed_boundaries && !cells.is_empty() {
        let h_edges = edges.get(&Orientation::Horizontal).unwrap();
        let v_edges = edges.get(&Orientation::Vertical).unwrap();
        let x_tol = tf_settings.intersection_x_tolerance.into_inner();
        let y_tol = tf_settings.intersection_y_tolerance.into_inner();

        // Group detected cells by table so that each table's boundary is checked
        // independently – prevents a shared extension x from merging two tables.
        let tables = cells_to_tables(&cells);
        let mut extra: Vec<BboxKey> = Vec::new();
        for table_cells in &tables {
            extra.extend(collect_unclosed_boundary_cells(
                table_cells,
                h_edges,
                v_edges,
                x_tol,
                y_tol,
                h_strat,
                v_strat,
            ));
        }
        cells.extend(extra);
    }

    cells
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
    let min_rows = tf_settings.and_then(|s| s.min_rows);
    let min_columns = tf_settings.and_then(|s| s.min_columns);
    let need_strip = tf_settings.is_none_or(|s| s.text_settings.need_strip);

    let tables_bbox = cells_to_tables(cells);
    let tables_bbox = filter_tables(tables_bbox, include_single_cell, min_rows, min_columns);

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
/// * `clip` - Optional clip region. If provided, only edges within this region
///   are used for table detection.
///
/// # Returns
///
/// A vector of Table objects found in the page.
pub fn find_tables(
    pdf_page: Option<&Page>,
    tf_settings: Rc<TfSettings>,
    extract_text: bool,
    clip: Option<&BboxKey>,
) -> Vec<Table> {
    let cells = find_all_cells_bboxes(pdf_page, tf_settings.clone(), clip);
    find_tables_from_cells(&cells, extract_text, pdf_page, Some(&tf_settings))
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
        let tables = cells_to_tables(&cells);
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
        let tables = cells_to_tables(&cells);
        assert_eq!(tables.len(), 2);
    }

    #[test]
    fn test_cells_to_tables_single_cell() {
        // cells_to_tables should not filter - single cell should be included
        let cells: Vec<BboxKey> = vec![(of(0.0), of(0.0), of(10.0), of(10.0))];
        let tables = cells_to_tables(&cells);
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].len(), 1);
    }

    #[test]
    fn test_cells_to_tables_empty() {
        let cells: Vec<BboxKey> = vec![];
        let tables = cells_to_tables(&cells);
        assert_eq!(tables.len(), 0);
    }

    #[test]
    fn test_filter_tables_single_cell_excluded() {
        let cells: Vec<BboxKey> = vec![(of(0.0), of(0.0), of(10.0), of(10.0))];
        let tables = cells_to_tables(&cells);
        let filtered = filter_tables(tables, false, None, None);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_tables_single_cell_included() {
        let cells: Vec<BboxKey> = vec![(of(0.0), of(0.0), of(10.0), of(10.0))];
        let tables = cells_to_tables(&cells);
        let filtered = filter_tables(tables, true, None, None);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_filter_tables_min_rows() {
        // Create a 2x2 table (2 rows, 2 cols)
        let cells: Vec<BboxKey> = vec![
            (of(0.0), of(0.0), of(10.0), of(10.0)),
            (of(10.0), of(0.0), of(20.0), of(10.0)),
            (of(0.0), of(10.0), of(10.0), of(20.0)),
            (of(10.0), of(10.0), of(20.0), of(20.0)),
        ];
        let tables = cells_to_tables(&cells);
        // min_rows=2 should keep the table
        let filtered = filter_tables(tables.clone(), false, Some(2), None);
        assert_eq!(filtered.len(), 1);
        // min_rows=3 should filter it out
        let filtered = filter_tables(tables, false, Some(3), None);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_tables_min_columns() {
        // Create a 2x2 table (2 rows, 2 cols)
        let cells: Vec<BboxKey> = vec![
            (of(0.0), of(0.0), of(10.0), of(10.0)),
            (of(10.0), of(0.0), of(20.0), of(10.0)),
            (of(0.0), of(10.0), of(10.0), of(20.0)),
            (of(10.0), of(10.0), of(20.0), of(20.0)),
        ];
        let tables = cells_to_tables(&cells);
        // min_columns=2 should keep the table
        let filtered = filter_tables(tables.clone(), false, Some(2), None);
        assert_eq!(filtered.len(), 1);
        // min_columns=3 should filter it out
        let filtered = filter_tables(tables, false, None, Some(3));
        assert_eq!(filtered.len(), 0);
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

    fn make_word(x1: f32, y1: f32, x2: f32, y2: f32, rotation: f32) -> Word {
        Word {
            text: "w".to_string(),
            bbox: (of(x1), of(y1), of(x2), of(y2)),
            rotation_degrees: of(rotation),
        }
    }

    #[test]
    fn test_word_gap_requires_space_ltr_gap_exceeds_tol() {
        // LTR (r=0°): prev ends at x=5, next starts at x=8 → gap=3 > x_tol=2
        let prev = make_word(0.0, 0.0, 5.0, 10.0, 0.0);
        let next = make_word(8.0, 0.0, 15.0, 10.0, 0.0);
        assert!(Table::word_gap_requires_space(&prev, &next, 2.0, 5.0));
    }

    #[test]
    fn test_word_gap_requires_space_ltr_gap_equals_tol_no_space() {
        // gap == tol is NOT strictly greater, so no space
        let prev = make_word(0.0, 0.0, 5.0, 10.0, 0.0);
        let next = make_word(8.0, 0.0, 15.0, 10.0, 0.0);
        assert!(!Table::word_gap_requires_space(&prev, &next, 3.0, 5.0));
    }

    #[test]
    fn test_word_gap_requires_space_ltr_wraparound_uses_x_tol() {
        // LTR wrap-around (r=320°): should use x_tol, not y_tol
        // gap=3 > x_tol=2 → true; gap=3 <= y_tol=5 would give false if wrong tol were used
        let prev = make_word(0.0, 0.0, 5.0, 10.0, 320.0);
        let next = make_word(8.0, 0.0, 15.0, 10.0, 320.0);
        assert!(Table::word_gap_requires_space(&prev, &next, 2.0, 5.0));
    }

    #[test]
    fn test_word_gap_requires_space_vertical_ttb_gap_exceeds_tol() {
        // Vertical top-to-bottom (r=90°): prev ends at y=5, next starts at y=8 → gap=3 > y_tol=2
        let prev = make_word(0.0, 0.0, 10.0, 5.0, 90.0);
        let next = make_word(0.0, 8.0, 10.0, 15.0, 90.0);
        assert!(Table::word_gap_requires_space(&prev, &next, 5.0, 2.0));
    }

    #[test]
    fn test_word_gap_requires_space_rtl_gap_exceeds_tol() {
        // RTL (r=180°): gap = prev.bbox.0 − next.bbox.2 = 8 − 5 = 3 > x_tol=2
        let prev = make_word(8.0, 0.0, 15.0, 10.0, 180.0);
        let next = make_word(0.0, 0.0, 5.0, 10.0, 180.0);
        assert!(Table::word_gap_requires_space(&prev, &next, 2.0, 5.0));
    }

    #[test]
    fn test_word_gap_requires_space_vertical_btt_gap_exceeds_tol() {
        // Vertical bottom-to-top (r=270°): gap = prev.bbox.1 − next.bbox.3 = 8 − 5 = 3 > y_tol=2
        let prev = make_word(0.0, 8.0, 10.0, 15.0, 270.0);
        let next = make_word(0.0, 0.0, 10.0, 5.0, 270.0);
        assert!(Table::word_gap_requires_space(&prev, &next, 5.0, 2.0));
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

    /// Helper: build a wide qualifying rect for tests.
    fn make_rect(x0: f32, y0: f32, x1: f32, y1: f32, r: u8, g: u8, b: u8) -> crate::objects::Rect {
        use pdfium_render::prelude::{PdfColor, PdfPathFillMode};
        crate::objects::Rect {
            bbox: (of(x0), of(y0), of(x1), of(y1)),
            fill_color: PdfColor::new(r, g, b, 255),
            stroke_color: PdfColor::new(0, 0, 0, 255),
            stroke_width: 0.0,
            is_stroked: false,
            fill_mode: PdfPathFillMode::Winding,
        }
    }

    #[test]
    fn test_invisible_edge_both_sides_same_color_excluded() {
        // H-edge (white) between two white rects → invisible → excluded
        use pdfium_render::prelude::PdfColor;
        let rects = vec![
            make_rect(0.0, 0.0, 100.0, 10.0, 255, 255, 255), // above, white
            make_rect(0.0, 10.0, 100.0, 20.0, 255, 255, 255), // below, white
        ];
        let mut edges = vec![Edge {
            orientation: Orientation::Horizontal,
            x1: of(0.0),
            y1: of(10.0),
            x2: of(100.0),
            y2: of(10.0),
            width: of(1.0),
            color: PdfColor::new(255, 255, 255, 255),
        }];
        filter_edges_invisible_against_background(&mut edges, &rects, 3.0, 3.0);
        assert!(
            edges.is_empty(),
            "white edge between two white rects should be excluded"
        );
    }

    #[test]
    fn test_visible_edge_sides_differ_kept() {
        // H-edge (white) between a blue rect and a green rect → visible → kept
        use pdfium_render::prelude::PdfColor;
        let rects = vec![
            make_rect(0.0, 0.0, 100.0, 10.0, 47, 84, 150), // above, blue
            make_rect(0.0, 10.0, 100.0, 20.0, 226, 239, 217), // below, green
        ];
        let mut edges = vec![Edge {
            orientation: Orientation::Horizontal,
            x1: of(0.0),
            y1: of(10.0),
            x2: of(100.0),
            y2: of(10.0),
            width: of(1.0),
            color: PdfColor::new(255, 255, 255, 255),
        }];
        filter_edges_invisible_against_background(&mut edges, &rects, 3.0, 3.0);
        assert_eq!(
            edges.len(),
            1,
            "white edge between blue and green rects should be kept"
        );
    }

    #[test]
    fn test_edge_one_side_white_on_white_excluded() {
        // White H-edge with only one adjacent white rect (other side = default page white).
        // The edge is invisible from both sides → excluded.
        // E.g. a white line at the top of a white-background row section.
        use pdfium_render::prelude::PdfColor;
        let rects = vec![make_rect(0.0, 10.0, 100.0, 30.0, 255, 255, 255)];
        let mut edges = vec![Edge {
            orientation: Orientation::Horizontal,
            x1: of(0.0),
            y1: of(10.0),
            x2: of(100.0),
            y2: of(10.0),
            width: of(1.0),
            color: PdfColor::new(255, 255, 255, 255),
        }];
        filter_edges_invisible_against_background(&mut edges, &rects, 3.0, 3.0);
        assert!(
            edges.is_empty(),
            "white edge adjacent to white rect (other side = page white) should be excluded"
        );
    }

    #[test]
    fn test_edge_one_side_white_border_on_colored_rect_kept() {
        // White H-edge with only one adjacent blue rect → kept.
        // The edge is visible against the blue background; from the other side the page
        // is white and the edge is also white, but from the blue side it is visible.
        // (Visible against at least one "effective" side ← blue ≠ white.)
        use pdfium_render::prelude::PdfColor;
        let rects = vec![make_rect(0.0, 10.0, 100.0, 30.0, 47, 84, 150)];
        let mut edges = vec![Edge {
            orientation: Orientation::Horizontal,
            x1: of(0.0),
            y1: of(10.0),
            x2: of(100.0),
            y2: of(10.0),
            width: of(1.0),
            color: PdfColor::new(255, 255, 255, 255), // white ≠ blue adjacent
        }];
        filter_edges_invisible_against_background(&mut edges, &rects, 3.0, 3.0);
        assert_eq!(
            edges.len(),
            1,
            "white edge adjacent to a colored rect should be kept (visible from rect side)"
        );
    }

    #[test]
    fn test_edge_one_side_non_white_on_same_color_rect_kept() {
        // Dark-red H-edge with only one adjacent dark-red rect (other side = page white).
        // The edge is invisible from the rect side but visible from the white-page side
        // (dark-red ≠ white) → kept.
        // E.g. the bottom border of the last dark-red row in a table.
        use pdfium_render::prelude::PdfColor;
        let rects = vec![make_rect(0.0, 0.0, 100.0, 10.0, 144, 12, 63)]; // dark-red above
        let mut edges = vec![Edge {
            orientation: Orientation::Horizontal,
            x1: of(0.0),
            y1: of(10.0),
            x2: of(100.0),
            y2: of(10.0),
            width: of(1.0),
            color: PdfColor::new(144, 12, 63, 255), // dark-red = same as adjacent rect
        }];
        filter_edges_invisible_against_background(&mut edges, &rects, 3.0, 3.0);
        assert_eq!(
            edges.len(),
            1,
            "non-white edge adjacent to same-color rect should be kept \
             (visible from default-white page side)"
        );
    }

    #[test]
    fn test_edge_inside_same_color_rect_excluded() {
        // H-edge (blue) with no adjacent rects but is contained inside a blue rect → excluded
        use pdfium_render::prelude::PdfColor;
        let rects = vec![
            make_rect(0.0, 0.0, 100.0, 50.0, 47, 84, 150), // large blue rect containing the edge
        ];
        let mut edges = vec![Edge {
            orientation: Orientation::Horizontal,
            x1: of(10.0),
            y1: of(25.0), // y=25 is inside the rect [0..50]
            x2: of(90.0),
            y2: of(25.0),
            width: of(0.5),
            color: PdfColor::new(47, 84, 150, 255), // same blue
        }];
        filter_edges_invisible_against_background(&mut edges, &rects, 3.0, 3.0);
        assert!(
            edges.is_empty(),
            "blue edge inside a blue rect should be excluded"
        );
    }

    #[test]
    fn test_edge_inside_different_color_rect_kept() {
        // H-edge (white) contained in a green rect → visible inside it → kept
        use pdfium_render::prelude::PdfColor;
        let rects = vec![make_rect(0.0, 0.0, 100.0, 50.0, 226, 239, 217)];
        let mut edges = vec![Edge {
            orientation: Orientation::Horizontal,
            x1: of(10.0),
            y1: of(25.0),
            x2: of(90.0),
            y2: of(25.0),
            width: of(1.0),
            color: PdfColor::new(255, 255, 255, 255), // white ≠ green
        }];
        filter_edges_invisible_against_background(&mut edges, &rects, 3.0, 3.0);
        assert_eq!(
            edges.len(),
            1,
            "white edge inside a green rect should be kept"
        );
    }

    #[test]
    fn test_no_context_white_edge_excluded() {
        // No rects at all, white edge → excluded (invisible on default white page)
        use pdfium_render::prelude::PdfColor;
        let rects: Vec<crate::objects::Rect> = vec![];
        let mut edges = vec![Edge {
            orientation: Orientation::Horizontal,
            x1: of(0.0),
            y1: of(10.0),
            x2: of(100.0),
            y2: of(10.0),
            width: of(1.0),
            color: PdfColor::new(255, 255, 255, 255),
        }];
        filter_edges_invisible_against_background(&mut edges, &rects, 3.0, 3.0);
        assert!(
            edges.is_empty(),
            "white edge with no context should be excluded"
        );
    }

    #[test]
    fn test_no_context_non_white_edge_kept() {
        // No rects at all, black edge → kept
        use pdfium_render::prelude::PdfColor;
        let rects: Vec<crate::objects::Rect> = vec![];
        let mut edges = vec![Edge {
            orientation: Orientation::Horizontal,
            x1: of(0.0),
            y1: of(10.0),
            x2: of(100.0),
            y2: of(10.0),
            width: of(1.0),
            color: PdfColor::new(0, 0, 0, 255),
        }];
        filter_edges_invisible_against_background(&mut edges, &rects, 3.0, 3.0);
        assert_eq!(
            edges.len(),
            1,
            "non-white edge with no context should be kept"
        );
    }

    #[test]
    fn test_vertical_edge_between_same_color_rects_excluded() {
        // V-edge (red) with red rects on both left and right → excluded
        use pdfium_render::prelude::PdfColor;
        let rects = vec![
            make_rect(0.0, 0.0, 10.0, 50.0, 200, 50, 50), // left, red
            make_rect(10.0, 0.0, 20.0, 50.0, 200, 50, 50), // right, red
        ];
        let mut edges = vec![Edge {
            orientation: Orientation::Vertical,
            x1: of(10.0),
            y1: of(0.0),
            x2: of(10.0),
            y2: of(50.0),
            width: of(1.0),
            color: PdfColor::new(200, 50, 50, 255),
        }];
        filter_edges_invisible_against_background(&mut edges, &rects, 3.0, 3.0);
        assert!(
            edges.is_empty(),
            "red V-edge between two red rects should be excluded"
        );
    }

    #[test]
    fn test_mixed_table_both_white_and_colored_bg() {
        // Simulates a page with two tables:
        //   - Table A: white cells, dark borders → dark H-edge between two white rects → kept
        //   - Table B: blue cells, white borders → white H-edge between two blue rects → kept
        use pdfium_render::prelude::PdfColor;
        let rects = vec![
            make_rect(0.0, 0.0, 50.0, 20.0, 255, 255, 255), // Table A, white cell above
            make_rect(0.0, 20.0, 50.0, 40.0, 255, 255, 255), // Table A, white cell below
            make_rect(60.0, 0.0, 110.0, 20.0, 47, 84, 150), // Table B, blue cell above
            make_rect(60.0, 20.0, 110.0, 40.0, 47, 84, 150), // Table B, blue cell below
        ];
        let mut edges = vec![
            Edge {
                // Table A: dark border between two white cells → kept (dark ≠ white)
                orientation: Orientation::Horizontal,
                x1: of(0.0),
                y1: of(20.0),
                x2: of(50.0),
                y2: of(20.0),
                width: of(1.0),
                color: PdfColor::new(50, 50, 50, 255),
            },
            Edge {
                // Table B: white border between two blue cells → kept (white ≠ blue)
                orientation: Orientation::Horizontal,
                x1: of(60.0),
                y1: of(20.0),
                x2: of(110.0),
                y2: of(20.0),
                width: of(1.0),
                color: PdfColor::new(255, 255, 255, 255),
            },
        ];
        filter_edges_invisible_against_background(&mut edges, &rects, 3.0, 3.0);
        assert_eq!(
            edges.len(),
            2,
            "both table borders should be kept in mixed-background page"
        );
    }

    #[test]
    fn test_filter_invisible_edges_empty_input() {
        let rects: Vec<crate::objects::Rect> = vec![];
        let mut edges: Vec<Edge> = vec![];
        filter_edges_invisible_against_background(&mut edges, &rects, 3.0, 3.0);
        assert_eq!(edges.len(), 0);
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
    fn test_to_vec_basic() {
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

        let vecs = table.to_vec().unwrap();
        assert_eq!(vecs.len(), 2);
        assert_eq!(
            vecs[0][0],
            TableCellValue {
                text: Some("A".to_string()),
                merged_left: false,
                merged_top: false,
            }
        );
        assert_eq!(vecs[0][1].text, Some("B".to_string()));
        assert!(vecs[1][0].text.as_deref() == Some("C"));
        assert!(vecs[1][1].text.as_deref() == Some("D"));
    }

    #[test]
    fn test_to_vec_with_empty_cells() {
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
        ];
        let table = Table {
            cells,
            bbox: (of(0.0), of(0.0), of(20.0), of(20.0)),
            page_index: 0,
            text_extracted: true,
        };

        let vecs = table.to_vec().unwrap();
        assert_eq!(vecs.len(), 2);
        assert_eq!(vecs[0][0].text.as_deref(), Some("abc "));
        assert_eq!(vecs[0][1].text.as_deref(), Some("q"));
        assert_eq!(vecs[1][0].text.as_deref(), Some(""));
        assert_eq!(vecs[1][1].text.as_deref(), Some("w"));
        assert!(!vecs[0][0].merged_left && !vecs[0][0].merged_top);
    }

    #[test]
    fn test_to_vec_with_merged_cells() {
        // Row 0: one cell spanning two columns (x 0..20); row 1: two cells
        let cells = vec![
            TableCell {
                text: "Merged".to_string(),
                bbox: (of(0.0), of(0.0), of(20.0), of(10.0)),
            },
            TableCell {
                text: "A".to_string(),
                bbox: (of(0.0), of(10.0), of(10.0), of(20.0)),
            },
            TableCell {
                text: "B".to_string(),
                bbox: (of(10.0), of(10.0), of(20.0), of(20.0)),
            },
        ];
        let table = Table {
            cells,
            bbox: (of(0.0), of(0.0), of(20.0), of(20.0)),
            page_index: 0,
            text_extracted: true,
        };

        let vecs = table.to_vec().unwrap();
        assert_eq!(vecs.len(), 2);
        // Row 0: first cell has text, second is merged from left
        assert_eq!(vecs[0][0].text.as_deref(), Some("Merged"));
        assert!(!vecs[0][0].merged_left && !vecs[0][0].merged_top);
        assert!(vecs[0][1].text.is_none());
        assert!(vecs[0][1].merged_left && !vecs[0][1].merged_top);
        // Row 1: both cells have text
        assert_eq!(vecs[1][0].text.as_deref(), Some("A"));
        assert_eq!(vecs[1][1].text.as_deref(), Some("B"));
    }

    #[test]
    fn test_to_vec_without_text_extracted() {
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

        let result = table.to_vec();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Text has not been extracted. Call extract_text first."
        );
    }

    #[test]
    fn test_py_table_cell_value_repr() {
        use super::py_table_cell_value_repr;
        // Text with content: double quotes around string
        assert_eq!(
            py_table_cell_value_repr(&Some("abc".to_string()), false, false),
            "(\"abc\", False, False)"
        );
        // None (merged cell)
        assert_eq!(
            py_table_cell_value_repr(&None, true, false),
            "(None, True, False)"
        );
        assert_eq!(
            py_table_cell_value_repr(&None, false, true),
            "(None, False, True)"
        );
        // String with double quote inside is escaped
        assert_eq!(
            py_table_cell_value_repr(&Some("say \"hi\"".to_string()), false, false),
            "(\"say \\\"hi\\\"\", False, False)"
        );
        // String with backslash is escaped
        assert_eq!(
            py_table_cell_value_repr(&Some("a\\b".to_string()), false, false),
            "(\"a\\\\b\", False, False)"
        );
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

    #[test]
    fn test_find_tables_with_explicit_edges_no_page() {
        use crate::edges::Edge;
        use crate::objects::Orientation;
        use crate::settings::{StrategyType, TfSettings};
        use pdfium_render::prelude::PdfColor;

        // Create a 2x2 grid using explicit edges
        let h_edges = vec![
            Edge {
                orientation: Orientation::Horizontal,
                x1: of(0.0),
                y1: of(0.0),
                x2: of(100.0),
                y2: of(0.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Horizontal,
                x1: of(0.0),
                y1: of(50.0),
                x2: of(100.0),
                y2: of(50.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Horizontal,
                x1: of(0.0),
                y1: of(100.0),
                x2: of(100.0),
                y2: of(100.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
        ];
        let v_edges = vec![
            Edge {
                orientation: Orientation::Vertical,
                x1: of(0.0),
                y1: of(0.0),
                x2: of(0.0),
                y2: of(100.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Vertical,
                x1: of(50.0),
                y1: of(0.0),
                x2: of(50.0),
                y2: of(100.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Vertical,
                x1: of(100.0),
                y1: of(0.0),
                x2: of(100.0),
                y2: of(100.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
        ];

        let settings = TfSettings {
            vertical_strategy: StrategyType::Explicit,
            horizontal_strategy: StrategyType::Explicit,
            explicit_h_edges: Some(h_edges),
            explicit_v_edges: Some(v_edges),
            ..Default::default()
        };

        // Call find_tables with page=None
        let tables = find_tables(None, Rc::new(settings), false, None);

        // 2x2 grid should produce 1 table with 4 cells
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].cells.len(), 4);
    }

    #[test]
    fn test_find_tables_with_explicit_edges_single_cell() {
        use crate::edges::Edge;
        use crate::objects::Orientation;
        use crate::settings::{StrategyType, TfSettings};
        use pdfium_render::prelude::PdfColor;

        // Create a single cell
        let h_edges = vec![
            Edge {
                orientation: Orientation::Horizontal,
                x1: of(0.0),
                y1: of(0.0),
                x2: of(100.0),
                y2: of(0.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Horizontal,
                x1: of(0.0),
                y1: of(100.0),
                x2: of(100.0),
                y2: of(100.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
        ];
        let v_edges = vec![
            Edge {
                orientation: Orientation::Vertical,
                x1: of(0.0),
                y1: of(0.0),
                x2: of(0.0),
                y2: of(100.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Vertical,
                x1: of(100.0),
                y1: of(0.0),
                x2: of(100.0),
                y2: of(100.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
        ];

        let settings = TfSettings {
            vertical_strategy: StrategyType::Explicit,
            horizontal_strategy: StrategyType::Explicit,
            explicit_h_edges: Some(h_edges),
            explicit_v_edges: Some(v_edges),
            include_single_cell: true,
            ..Default::default()
        };

        let tables = find_tables(None, Rc::new(settings), false, None);

        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].cells.len(), 1);
    }

    #[test]
    fn test_find_tables_with_empty_explicit_edges() {
        use crate::settings::{StrategyType, TfSettings};

        let settings = TfSettings {
            vertical_strategy: StrategyType::Explicit,
            horizontal_strategy: StrategyType::Explicit,
            explicit_h_edges: Some(vec![]),
            explicit_v_edges: Some(vec![]),
            ..Default::default()
        };

        let tables = find_tables(None, Rc::new(settings), false, None);

        assert_eq!(tables.len(), 0);
    }

    #[test]
    fn test_find_all_cells_bboxes_with_explicit_edges_no_page() {
        use crate::edges::Edge;
        use crate::objects::Orientation;
        use crate::settings::{StrategyType, TfSettings};
        use pdfium_render::prelude::PdfColor;

        // Create a 2x2 grid
        let h_edges = vec![
            Edge {
                orientation: Orientation::Horizontal,
                x1: of(0.0),
                y1: of(0.0),
                x2: of(100.0),
                y2: of(0.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Horizontal,
                x1: of(0.0),
                y1: of(50.0),
                x2: of(100.0),
                y2: of(50.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Horizontal,
                x1: of(0.0),
                y1: of(100.0),
                x2: of(100.0),
                y2: of(100.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
        ];
        let v_edges = vec![
            Edge {
                orientation: Orientation::Vertical,
                x1: of(0.0),
                y1: of(0.0),
                x2: of(0.0),
                y2: of(100.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Vertical,
                x1: of(50.0),
                y1: of(0.0),
                x2: of(50.0),
                y2: of(100.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Vertical,
                x1: of(100.0),
                y1: of(0.0),
                x2: of(100.0),
                y2: of(100.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
        ];

        let settings = TfSettings {
            vertical_strategy: StrategyType::Explicit,
            horizontal_strategy: StrategyType::Explicit,
            explicit_h_edges: Some(h_edges),
            explicit_v_edges: Some(v_edges),
            ..Default::default()
        };

        let cells = find_all_cells_bboxes(None, Rc::new(settings), None);

        // 2x2 grid should produce 4 cells
        assert_eq!(cells.len(), 4);
    }

    #[test]
    fn test_find_tables_3x3_grid_no_page() {
        use crate::edges::Edge;
        use crate::objects::Orientation;
        use crate::settings::{StrategyType, TfSettings};
        use pdfium_render::prelude::PdfColor;

        // Create a 3x3 grid
        let h_edges = vec![
            Edge {
                orientation: Orientation::Horizontal,
                x1: of(0.0),
                y1: of(0.0),
                x2: of(150.0),
                y2: of(0.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Horizontal,
                x1: of(0.0),
                y1: of(50.0),
                x2: of(150.0),
                y2: of(50.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Horizontal,
                x1: of(0.0),
                y1: of(100.0),
                x2: of(150.0),
                y2: of(100.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Horizontal,
                x1: of(0.0),
                y1: of(150.0),
                x2: of(150.0),
                y2: of(150.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
        ];
        let v_edges = vec![
            Edge {
                orientation: Orientation::Vertical,
                x1: of(0.0),
                y1: of(0.0),
                x2: of(0.0),
                y2: of(150.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Vertical,
                x1: of(50.0),
                y1: of(0.0),
                x2: of(50.0),
                y2: of(150.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Vertical,
                x1: of(100.0),
                y1: of(0.0),
                x2: of(100.0),
                y2: of(150.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Vertical,
                x1: of(150.0),
                y1: of(0.0),
                x2: of(150.0),
                y2: of(150.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
        ];

        let settings = TfSettings {
            vertical_strategy: StrategyType::Explicit,
            horizontal_strategy: StrategyType::Explicit,
            explicit_h_edges: Some(h_edges),
            explicit_v_edges: Some(v_edges),
            ..Default::default()
        };

        let tables = find_tables(None, Rc::new(settings), false, None);

        assert_eq!(tables.len(), 1);
        // 3x3 grid should have 9 cells
        assert_eq!(tables[0].cells.len(), 9);
    }

    #[test]
    fn test_clip_edges_to_bbox_horizontal_edges() {
        use crate::edges::Edge;
        use pdfium_render::prelude::PdfColor;

        let mut edges = HashMap::new();
        edges.insert(
            Orientation::Horizontal,
            vec![
                // Edge completely inside clip region
                Edge {
                    orientation: Orientation::Horizontal,
                    x1: of(20.0),
                    y1: of(50.0),
                    x2: of(80.0),
                    y2: of(50.0),
                    width: of(1.0),
                    color: PdfColor::new(0, 0, 0, 255),
                },
                // Edge crossing clip region (should be clipped)
                Edge {
                    orientation: Orientation::Horizontal,
                    x1: of(0.0),
                    y1: of(60.0),
                    x2: of(150.0),
                    y2: of(60.0),
                    width: of(1.0),
                    color: PdfColor::new(0, 0, 0, 255),
                },
                // Edge completely outside clip region (y outside)
                Edge {
                    orientation: Orientation::Horizontal,
                    x1: of(20.0),
                    y1: of(150.0),
                    x2: of(80.0),
                    y2: of(150.0),
                    width: of(1.0),
                    color: PdfColor::new(0, 0, 0, 255),
                },
                // Edge completely outside clip region (x outside)
                Edge {
                    orientation: Orientation::Horizontal,
                    x1: of(110.0),
                    y1: of(50.0),
                    x2: of(150.0),
                    y2: of(50.0),
                    width: of(1.0),
                    color: PdfColor::new(0, 0, 0, 255),
                },
            ],
        );
        edges.insert(Orientation::Vertical, vec![]);

        let clip: BboxKey = (of(10.0), of(10.0), of(100.0), of(100.0));
        clip_edges_to_bbox(&mut edges, &clip);

        let h_edges = edges.get(&Orientation::Horizontal).unwrap();
        assert_eq!(h_edges.len(), 2);

        // First edge should be unchanged (was already inside)
        assert_eq!(h_edges[0].x1, of(20.0));
        assert_eq!(h_edges[0].x2, of(80.0));
        assert_eq!(h_edges[0].y1, of(50.0));

        // Second edge should be clipped to clip region
        assert_eq!(h_edges[1].x1, of(10.0)); // Clipped from 0.0
        assert_eq!(h_edges[1].x2, of(100.0)); // Clipped from 150.0
        assert_eq!(h_edges[1].y1, of(60.0));
    }

    #[test]
    fn test_clip_edges_to_bbox_vertical_edges() {
        use crate::edges::Edge;
        use pdfium_render::prelude::PdfColor;

        let mut edges = HashMap::new();
        edges.insert(Orientation::Horizontal, vec![]);
        edges.insert(
            Orientation::Vertical,
            vec![
                // Edge completely inside clip region
                Edge {
                    orientation: Orientation::Vertical,
                    x1: of(50.0),
                    y1: of(20.0),
                    x2: of(50.0),
                    y2: of(80.0),
                    width: of(1.0),
                    color: PdfColor::new(0, 0, 0, 255),
                },
                // Edge crossing clip region (should be clipped)
                Edge {
                    orientation: Orientation::Vertical,
                    x1: of(60.0),
                    y1: of(0.0),
                    x2: of(60.0),
                    y2: of(150.0),
                    width: of(1.0),
                    color: PdfColor::new(0, 0, 0, 255),
                },
                // Edge completely outside clip region (x outside)
                Edge {
                    orientation: Orientation::Vertical,
                    x1: of(150.0),
                    y1: of(20.0),
                    x2: of(150.0),
                    y2: of(80.0),
                    width: of(1.0),
                    color: PdfColor::new(0, 0, 0, 255),
                },
                // Edge completely outside clip region (y outside)
                Edge {
                    orientation: Orientation::Vertical,
                    x1: of(50.0),
                    y1: of(110.0),
                    x2: of(50.0),
                    y2: of(150.0),
                    width: of(1.0),
                    color: PdfColor::new(0, 0, 0, 255),
                },
            ],
        );

        let clip: BboxKey = (of(10.0), of(10.0), of(100.0), of(100.0));
        clip_edges_to_bbox(&mut edges, &clip);

        let v_edges = edges.get(&Orientation::Vertical).unwrap();
        assert_eq!(v_edges.len(), 2);

        // First edge should be unchanged (was already inside)
        assert_eq!(v_edges[0].x1, of(50.0));
        assert_eq!(v_edges[0].y1, of(20.0));
        assert_eq!(v_edges[0].y2, of(80.0));

        // Second edge should be clipped to clip region
        assert_eq!(v_edges[1].x1, of(60.0));
        assert_eq!(v_edges[1].y1, of(10.0)); // Clipped from 0.0
        assert_eq!(v_edges[1].y2, of(100.0)); // Clipped from 150.0
    }

    #[test]
    fn test_clip_edges_to_bbox_empty_result() {
        use crate::edges::Edge;
        use pdfium_render::prelude::PdfColor;

        let mut edges = HashMap::new();
        edges.insert(
            Orientation::Horizontal,
            vec![Edge {
                orientation: Orientation::Horizontal,
                x1: of(200.0),
                y1: of(200.0),
                x2: of(300.0),
                y2: of(200.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            }],
        );
        edges.insert(
            Orientation::Vertical,
            vec![Edge {
                orientation: Orientation::Vertical,
                x1: of(200.0),
                y1: of(200.0),
                x2: of(200.0),
                y2: of(300.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            }],
        );

        let clip: BboxKey = (of(0.0), of(0.0), of(100.0), of(100.0));
        clip_edges_to_bbox(&mut edges, &clip);

        // All edges should be removed
        assert_eq!(edges.get(&Orientation::Horizontal).unwrap().len(), 0);
        assert_eq!(edges.get(&Orientation::Vertical).unwrap().len(), 0);
    }

    #[test]
    fn test_find_tables_with_clip() {
        use crate::edges::Edge;
        use crate::settings::{StrategyType, TfSettings};
        use pdfium_render::prelude::PdfColor;

        // Create a 3x3 grid (150x150)
        let h_edges = vec![
            Edge {
                orientation: Orientation::Horizontal,
                x1: of(0.0),
                y1: of(0.0),
                x2: of(150.0),
                y2: of(0.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Horizontal,
                x1: of(0.0),
                y1: of(50.0),
                x2: of(150.0),
                y2: of(50.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Horizontal,
                x1: of(0.0),
                y1: of(100.0),
                x2: of(150.0),
                y2: of(100.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Horizontal,
                x1: of(0.0),
                y1: of(150.0),
                x2: of(150.0),
                y2: of(150.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
        ];
        let v_edges = vec![
            Edge {
                orientation: Orientation::Vertical,
                x1: of(0.0),
                y1: of(0.0),
                x2: of(0.0),
                y2: of(150.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Vertical,
                x1: of(50.0),
                y1: of(0.0),
                x2: of(50.0),
                y2: of(150.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Vertical,
                x1: of(100.0),
                y1: of(0.0),
                x2: of(100.0),
                y2: of(150.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Vertical,
                x1: of(150.0),
                y1: of(0.0),
                x2: of(150.0),
                y2: of(150.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
        ];

        let settings = TfSettings {
            vertical_strategy: StrategyType::Explicit,
            horizontal_strategy: StrategyType::Explicit,
            explicit_h_edges: Some(h_edges),
            explicit_v_edges: Some(v_edges),
            ..Default::default()
        };

        // Without clip: should have 9 cells (3x3 grid)
        let tables_no_clip = find_tables(None, Rc::new(settings.clone()), false, None);
        assert_eq!(tables_no_clip.len(), 1);
        assert_eq!(tables_no_clip[0].cells.len(), 9);

        // With clip to get only 2x2 grid (top-left corner)
        let clip: BboxKey = (of(0.0), of(0.0), of(100.0), of(100.0));
        let tables_with_clip = find_tables(None, Rc::new(settings), false, Some(&clip));
        assert_eq!(tables_with_clip.len(), 1);
        assert_eq!(tables_with_clip[0].cells.len(), 4); // 2x2 = 4 cells
    }

    #[test]
    fn test_find_all_cells_bboxes_with_clip() {
        use crate::edges::Edge;
        use crate::settings::{StrategyType, TfSettings};
        use pdfium_render::prelude::PdfColor;

        // Create a 2x2 grid (100x100)
        let h_edges = vec![
            Edge {
                orientation: Orientation::Horizontal,
                x1: of(0.0),
                y1: of(0.0),
                x2: of(100.0),
                y2: of(0.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Horizontal,
                x1: of(0.0),
                y1: of(50.0),
                x2: of(100.0),
                y2: of(50.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Horizontal,
                x1: of(0.0),
                y1: of(100.0),
                x2: of(100.0),
                y2: of(100.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
        ];
        let v_edges = vec![
            Edge {
                orientation: Orientation::Vertical,
                x1: of(0.0),
                y1: of(0.0),
                x2: of(0.0),
                y2: of(100.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Vertical,
                x1: of(50.0),
                y1: of(0.0),
                x2: of(50.0),
                y2: of(100.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Vertical,
                x1: of(100.0),
                y1: of(0.0),
                x2: of(100.0),
                y2: of(100.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
        ];

        let settings = TfSettings {
            vertical_strategy: StrategyType::Explicit,
            horizontal_strategy: StrategyType::Explicit,
            explicit_h_edges: Some(h_edges),
            explicit_v_edges: Some(v_edges),
            ..Default::default()
        };

        // Without clip: should have 4 cells
        let cells_no_clip = find_all_cells_bboxes(None, Rc::new(settings.clone()), None);
        assert_eq!(cells_no_clip.len(), 4);

        // With clip to get only 1 cell (top-left corner)
        let clip: BboxKey = (of(0.0), of(0.0), of(50.0), of(50.0));
        let cells_with_clip = find_all_cells_bboxes(None, Rc::new(settings), Some(&clip));
        assert_eq!(cells_with_clip.len(), 1);
        // Verify the cell bbox
        assert_eq!(cells_with_clip[0].0, of(0.0)); // x1
        assert_eq!(cells_with_clip[0].1, of(0.0)); // y1
        assert_eq!(cells_with_clip[0].2, of(50.0)); // x2
        assert_eq!(cells_with_clip[0].3, of(50.0)); // y2
    }

    #[test]
    fn test_find_tables_with_clip_no_tables() {
        use crate::edges::Edge;
        use crate::settings::{StrategyType, TfSettings};
        use pdfium_render::prelude::PdfColor;

        // Create a simple 2x2 grid at position (100, 100) to (200, 200)
        let h_edges = vec![
            Edge {
                orientation: Orientation::Horizontal,
                x1: of(100.0),
                y1: of(100.0),
                x2: of(200.0),
                y2: of(100.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Horizontal,
                x1: of(100.0),
                y1: of(150.0),
                x2: of(200.0),
                y2: of(150.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Horizontal,
                x1: of(100.0),
                y1: of(200.0),
                x2: of(200.0),
                y2: of(200.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
        ];
        let v_edges = vec![
            Edge {
                orientation: Orientation::Vertical,
                x1: of(100.0),
                y1: of(100.0),
                x2: of(100.0),
                y2: of(200.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Vertical,
                x1: of(150.0),
                y1: of(100.0),
                x2: of(150.0),
                y2: of(200.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
            Edge {
                orientation: Orientation::Vertical,
                x1: of(200.0),
                y1: of(100.0),
                x2: of(200.0),
                y2: of(200.0),
                width: of(1.0),
                color: PdfColor::new(0, 0, 0, 255),
            },
        ];

        let settings = TfSettings {
            vertical_strategy: StrategyType::Explicit,
            horizontal_strategy: StrategyType::Explicit,
            explicit_h_edges: Some(h_edges),
            explicit_v_edges: Some(v_edges),
            ..Default::default()
        };

        // Clip region that doesn't intersect with the table
        let clip: BboxKey = (of(0.0), of(0.0), of(50.0), of(50.0));
        let tables = find_tables(None, Rc::new(settings), false, Some(&clip));
        assert_eq!(tables.len(), 0);
    }

    #[test]
    fn test_clip_edges_partial_intersection() {
        use crate::edges::Edge;
        use pdfium_render::prelude::PdfColor;

        let mut edges = HashMap::new();
        edges.insert(
            Orientation::Horizontal,
            vec![
                // Edge partially overlapping on left side
                Edge {
                    orientation: Orientation::Horizontal,
                    x1: of(0.0),
                    y1: of(50.0),
                    x2: of(60.0),
                    y2: of(50.0),
                    width: of(1.0),
                    color: PdfColor::new(0, 0, 0, 255),
                },
                // Edge partially overlapping on right side
                Edge {
                    orientation: Orientation::Horizontal,
                    x1: of(80.0),
                    y1: of(50.0),
                    x2: of(150.0),
                    y2: of(50.0),
                    width: of(1.0),
                    color: PdfColor::new(0, 0, 0, 255),
                },
            ],
        );
        edges.insert(
            Orientation::Vertical,
            vec![
                // Edge partially overlapping on top side
                Edge {
                    orientation: Orientation::Vertical,
                    x1: of(50.0),
                    y1: of(0.0),
                    x2: of(50.0),
                    y2: of(60.0),
                    width: of(1.0),
                    color: PdfColor::new(0, 0, 0, 255),
                },
                // Edge partially overlapping on bottom side
                Edge {
                    orientation: Orientation::Vertical,
                    x1: of(50.0),
                    y1: of(80.0),
                    x2: of(50.0),
                    y2: of(150.0),
                    width: of(1.0),
                    color: PdfColor::new(0, 0, 0, 255),
                },
            ],
        );

        let clip: BboxKey = (of(30.0), of(30.0), of(100.0), of(100.0));
        clip_edges_to_bbox(&mut edges, &clip);

        let h_edges = edges.get(&Orientation::Horizontal).unwrap();
        assert_eq!(h_edges.len(), 2);
        // First edge: x1 clipped from 0 to 30
        assert_eq!(h_edges[0].x1, of(30.0));
        assert_eq!(h_edges[0].x2, of(60.0));
        // Second edge: x2 clipped from 150 to 100
        assert_eq!(h_edges[1].x1, of(80.0));
        assert_eq!(h_edges[1].x2, of(100.0));

        let v_edges = edges.get(&Orientation::Vertical).unwrap();
        assert_eq!(v_edges.len(), 2);
        // First edge: y1 clipped from 0 to 30
        assert_eq!(v_edges[0].y1, of(30.0));
        assert_eq!(v_edges[0].y2, of(60.0));
        // Second edge: y2 clipped from 150 to 100
        assert_eq!(v_edges[1].y1, of(80.0));
        assert_eq!(v_edges[1].y2, of(100.0));
    }

    // ── get_intersections_from_edges ────────────────────────────────────────

    fn make_h_edge(x1: f32, y: f32, x2: f32) -> Edge {
        use pdfium_render::prelude::PdfColor;
        Edge {
            orientation: Orientation::Horizontal,
            x1: of(x1),
            y1: of(y),
            x2: of(x2),
            y2: of(y),
            width: of(1.0),
            color: PdfColor::new(0, 0, 0, 255),
        }
    }

    fn make_v_edge(x: f32, y1: f32, y2: f32) -> Edge {
        use pdfium_render::prelude::PdfColor;
        Edge {
            orientation: Orientation::Vertical,
            x1: of(x),
            y1: of(y1),
            x2: of(x),
            y2: of(y2),
            width: of(1.0),
            color: PdfColor::new(0, 0, 0, 255),
        }
    }

    #[test]
    fn test_get_intersections_from_edges_single_crossing() {
        // One h-edge crossing one v-edge → exactly one intersection point.
        let h = vec![make_h_edge(0.0, 50.0, 100.0)];
        let v = vec![make_v_edge(50.0, 0.0, 100.0)];

        let settings = Rc::new(TfSettings::default());
        let intersections = TableFinder::new(settings).get_intersections_from_edges(h, v);

        assert_eq!(intersections.len(), 1);
        let point = (of(50.0), of(50.0));
        assert!(intersections.contains_key(&point));
    }

    #[test]
    fn test_get_intersections_from_edges_grid_2x2() {
        // 3 h-edges × 3 v-edges form a 2×2 grid → 9 intersection points.
        let h = vec![
            make_h_edge(0.0, 0.0, 100.0),
            make_h_edge(0.0, 50.0, 100.0),
            make_h_edge(0.0, 100.0, 100.0),
        ];
        let v = vec![
            make_v_edge(0.0, 0.0, 100.0),
            make_v_edge(50.0, 0.0, 100.0),
            make_v_edge(100.0, 0.0, 100.0),
        ];

        let settings = Rc::new(TfSettings::default());
        let intersections = TableFinder::new(settings).get_intersections_from_edges(h, v);

        assert_eq!(intersections.len(), 9);
        // Spot-check a few corners
        assert!(intersections.contains_key(&(of(0.0), of(0.0))));
        assert!(intersections.contains_key(&(of(100.0), of(100.0))));
        assert!(intersections.contains_key(&(of(50.0), of(50.0))));
    }

    #[test]
    fn test_get_intersections_from_edges_empty_input() {
        // No edges → no intersections.
        let settings = Rc::new(TfSettings::default());
        let intersections = TableFinder::new(settings).get_intersections_from_edges(vec![], vec![]);

        assert!(intersections.is_empty());
    }

    #[test]
    fn test_get_intersections_from_edges_no_crossing() {
        // Parallel edges that never cross → no intersections.
        let h = vec![make_h_edge(0.0, 50.0, 40.0)]; // ends at x=40
        let v = vec![make_v_edge(60.0, 0.0, 100.0)]; // starts at x=60

        let settings = Rc::new(TfSettings::default());
        let intersections = TableFinder::new(settings).get_intersections_from_edges(h, v);

        assert!(intersections.is_empty());
    }

    #[test]
    fn test_get_intersections_from_edges_only_h_edges() {
        // Only horizontal edges, no vertical → no intersections.
        let h = vec![make_h_edge(0.0, 0.0, 100.0), make_h_edge(0.0, 50.0, 100.0)];

        let settings = Rc::new(TfSettings::default());
        let intersections = TableFinder::new(settings).get_intersections_from_edges(h, vec![]);

        assert!(intersections.is_empty());
    }

    #[test]
    fn test_get_intersections_from_edges_point_coordinates() {
        // Verify the exact (x, y) coordinates of the intersection point.
        // h-edge at y=30, from x=10 to x=90
        // v-edge at x=70, from y=10 to y=80
        let h = vec![make_h_edge(10.0, 30.0, 90.0)];
        let v = vec![make_v_edge(70.0, 10.0, 80.0)];

        let settings = Rc::new(TfSettings::default());
        let intersections = TableFinder::new(settings).get_intersections_from_edges(h, v);

        assert_eq!(intersections.len(), 1);
        // The intersection point must be (v.x1, h.y1) = (70, 30)
        let point = (of(70.0), of(30.0));
        assert!(intersections.contains_key(&point));
    }
}
