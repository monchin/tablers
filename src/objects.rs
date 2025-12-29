use ordered_float::OrderedFloat;
use pdfium_render::prelude::PdfColor;
use pyo3::prelude::*;

/// Container for all extracted objects from a PDF page.
///
/// This struct holds all rectangles, lines, and characters found in a page.
#[pyclass]
#[derive(Clone)]
pub struct Objects {
    /// All rectangles found in the page.
    #[pyo3(get)]
    pub rects: Vec<Rect>,
    /// All line segments found in the page.
    #[pyo3(get)]
    pub lines: Vec<Line>,
    /// All text characters found in the page.
    #[pyo3(get)]
    pub chars: Vec<Char>,
}

/// A 2D point represented as (x, y) coordinates.
pub type Point = (OrderedFloat<f32>, OrderedFloat<f32>);

/// A bounding box key represented as (x1, y1, x2, y2) coordinates.
///
/// - x1: left edge
/// - y1: top edge
/// - x2: right edge
/// - y2: bottom edge
pub type BboxKey = (
    OrderedFloat<f32>,
    OrderedFloat<f32>,
    OrderedFloat<f32>,
    OrderedFloat<f32>,
);
/// Represents a rectangle extracted from a PDF page.
///
/// Rectangles are typically used as table cell borders or backgrounds.
#[pyclass]
#[derive(Clone)]
pub struct Rect {
    /// The bounding box of the rectangle.
    pub bbox: BboxKey,
    /// The fill color of the rectangle.
    pub fill_color: PdfColor,
    /// The stroke (border) color of the rectangle.
    pub stroke_color: PdfColor,
    /// The stroke width of the rectangle border.
    #[pyo3(get)]
    pub stroke_width: f32,
}

#[pymethods]
impl Rect {
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

    /// Returns the fill color as an RGBA tuple.
    #[getter]
    fn fill_color(&self) -> (u8, u8, u8, u8) {
        (
            self.fill_color.red(),
            self.fill_color.green(),
            self.fill_color.blue(),
            self.fill_color.alpha(),
        )
    }

    /// Returns the stroke color as an RGBA tuple.
    #[getter]
    fn stroke_color(&self) -> (u8, u8, u8, u8) {
        (
            self.stroke_color.red(),
            self.stroke_color.green(),
            self.stroke_color.blue(),
            self.stroke_color.alpha(),
        )
    }
}

/// Represents a line segment extracted from a PDF page.
///
/// Lines can be straight or curved and are used for table borders.
#[pyclass]
#[derive(Clone)]
pub struct Line {
    /// The type of line (straight or curve).
    pub line_type: LineType,
    /// The points that define the line path.
    pub points: Vec<Point>,
    /// The color of the line.
    pub color: PdfColor,
    /// The width of the line stroke.
    pub width: OrderedFloat<f32>,
}

#[pymethods]
impl Line {
    /// Returns the line type as a string ("straight" or "curve").
    #[getter]
    fn line_type(&self) -> &str {
        match self.line_type {
            LineType::Straight => "straight",
            LineType::Curve => "curve",
        }
    }

    /// Returns the line points as a list of (x, y) tuples.
    #[getter]
    fn points(&self) -> Vec<(f32, f32)> {
        self.points
            .iter()
            .map(|p| (p.0.into_inner(), p.1.into_inner()))
            .collect()
    }

    /// Returns the line color as an RGBA tuple.
    #[getter]
    fn color(&self) -> (u8, u8, u8, u8) {
        (
            self.color.red(),
            self.color.green(),
            self.color.blue(),
            self.color.alpha(),
        )
    }

    /// Returns the line width.
    #[getter]
    fn width(&self) -> f32 {
        self.width.into_inner()
    }
}

/// Represents a text character extracted from a PDF page.
///
/// Each character includes its Unicode value, position, and rotation information.
#[pyclass]
#[derive(Debug, Clone)]
pub struct Char {
    /// The Unicode string representation of the character.
    #[pyo3(get)]
    pub unicode_char: Option<String>,
    /// The bounding box of the character.
    pub bbox: BboxKey,
    /// The clockwise rotation of the character in degrees.
    pub rotation_degrees: OrderedFloat<f32>,
    /// Whether the character is upright (horizontal text).
    #[pyo3(get)]
    pub upright: bool,
}
#[pymethods]
impl Char {
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

    /// Returns the rotation of the character in degrees.
    #[getter]
    fn rotation_degrees(&self) -> f32 {
        self.rotation_degrees.into_inner()
    }
}

impl HasBbox for Char {
    fn bbox(&self) -> BboxKey {
        self.bbox
    }
}

/// Represents the orientation of a line or edge.
#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum Orientation {
    /// A vertical line (top to bottom).
    Vertical,
    /// A horizontal line (left to right).
    Horizontal,
}

/// Represents the type of a line segment.
#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum LineType {
    /// A straight line between two points.
    Straight,
    /// A curved line (Bezier curve).
    Curve,
}

/// Checks if a set of points forms a rectangle.
///
/// A valid rectangle has 5 points (4 corners + closing point) where
/// the first and last points are the same.
///
/// # Arguments
///
/// * `points` - A slice of points to check.
///
/// # Returns
///
/// `true` if the points form a rectangle, `false` otherwise.
pub(crate) fn is_rect(points: &[Point]) -> bool {
    if points.len() != 5 || points[0] != points[4] {
        return false;
    }
    if points[0].0 == points[1].0
        && points[1].1 == points[2].1
        && points[2].0 == points[3].0
        && points[3].1 == points[0].1
    {
        return true;
    }
    if points[0].1 == points[1].1
        && points[1].0 == points[2].0
        && points[2].1 == points[3].1
        && points[3].0 == points[0].0
    {
        return true;
    }
    false
}

/// Trait for objects that have a bounding box.
pub(crate) trait HasBbox {
    /// Returns the bounding box of the object.
    fn bbox(&self) -> BboxKey;
}

/// Merges multiple bounding boxes into one that encompasses all of them.
///
/// # Arguments
///
/// * `bboxes` - An iterator of bounding boxes to merge.
///
/// # Returns
///
/// The merged bounding box, or `None` if the iterator is empty.
fn merge_bboxes(bboxes: impl Iterator<Item = BboxKey>) -> Option<BboxKey> {
    bboxes.fold(None, |acc, (x1, y1, x2, y2)| {
        Some(match acc {
            None => (x1, y1, x2, y2),
            Some((ax1, ay1, ax2, ay2)) => (ax1.min(x1), ay1.min(y1), ax2.max(x2), ay2.max(y2)),
        })
    })
}

/// Gets the combined bounding box of multiple objects.
///
/// # Arguments
///
/// * `objects` - A slice of objects that implement HasBbox.
///
/// # Returns
///
/// The combined bounding box, or `None` if the slice is empty.
pub(crate) fn get_objects_bbox<T: HasBbox>(objects: &[T]) -> Option<BboxKey> {
    merge_bboxes(objects.iter().map(|obj| obj.bbox()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ordered_float::OrderedFloat;

    #[test]
    fn test_is_rect_valid_rectangle_clockwise() {
        // A valid rectangle with 5 points (first == last), clockwise
        let points: Vec<Point> = vec![
            (OrderedFloat(0.0), OrderedFloat(0.0)),
            (OrderedFloat(0.0), OrderedFloat(10.0)),
            (OrderedFloat(10.0), OrderedFloat(10.0)),
            (OrderedFloat(10.0), OrderedFloat(0.0)),
            (OrderedFloat(0.0), OrderedFloat(0.0)),
        ];
        assert!(is_rect(&points));
    }

    #[test]
    fn test_is_rect_valid_rectangle_counterclockwise() {
        // A valid rectangle with 5 points, counterclockwise order
        let points: Vec<Point> = vec![
            (OrderedFloat(0.0), OrderedFloat(0.0)),
            (OrderedFloat(10.0), OrderedFloat(0.0)),
            (OrderedFloat(10.0), OrderedFloat(10.0)),
            (OrderedFloat(0.0), OrderedFloat(10.0)),
            (OrderedFloat(0.0), OrderedFloat(0.0)),
        ];
        assert!(is_rect(&points));
    }

    #[test]
    fn test_is_rect_invalid_not_closed() {
        // 5 points but first != last
        let points: Vec<Point> = vec![
            (OrderedFloat(0.0), OrderedFloat(0.0)),
            (OrderedFloat(0.0), OrderedFloat(10.0)),
            (OrderedFloat(10.0), OrderedFloat(10.0)),
            (OrderedFloat(10.0), OrderedFloat(0.0)),
            (OrderedFloat(1.0), OrderedFloat(0.0)), // different from first
        ];
        assert!(!is_rect(&points));
    }

    #[test]
    fn test_is_rect_invalid_wrong_count() {
        // Only 4 points
        let points: Vec<Point> = vec![
            (OrderedFloat(0.0), OrderedFloat(0.0)),
            (OrderedFloat(0.0), OrderedFloat(10.0)),
            (OrderedFloat(10.0), OrderedFloat(10.0)),
            (OrderedFloat(10.0), OrderedFloat(0.0)),
        ];
        assert!(!is_rect(&points));
    }

    #[test]
    fn test_is_rect_invalid_not_rectangular() {
        // 5 points but not forming a rectangle
        let points: Vec<Point> = vec![
            (OrderedFloat(0.0), OrderedFloat(0.0)),
            (OrderedFloat(5.0), OrderedFloat(10.0)),
            (OrderedFloat(10.0), OrderedFloat(10.0)),
            (OrderedFloat(10.0), OrderedFloat(0.0)),
            (OrderedFloat(0.0), OrderedFloat(0.0)),
        ];
        assert!(!is_rect(&points));
    }

    #[test]
    fn test_merge_bboxes_single() {
        let bboxes = vec![(
            OrderedFloat(1.0),
            OrderedFloat(2.0),
            OrderedFloat(3.0),
            OrderedFloat(4.0),
        )];
        let result = merge_bboxes(bboxes.into_iter());
        assert_eq!(
            result,
            Some((
                OrderedFloat(1.0),
                OrderedFloat(2.0),
                OrderedFloat(3.0),
                OrderedFloat(4.0)
            ))
        );
    }

    #[test]
    fn test_merge_bboxes_multiple() {
        let bboxes = vec![
            (
                OrderedFloat(0.0),
                OrderedFloat(0.0),
                OrderedFloat(10.0),
                OrderedFloat(10.0),
            ),
            (
                OrderedFloat(5.0),
                OrderedFloat(5.0),
                OrderedFloat(15.0),
                OrderedFloat(15.0),
            ),
        ];
        let result = merge_bboxes(bboxes.into_iter());
        assert_eq!(
            result,
            Some((
                OrderedFloat(0.0),
                OrderedFloat(0.0),
                OrderedFloat(15.0),
                OrderedFloat(15.0)
            ))
        );
    }

    #[test]
    fn test_merge_bboxes_empty() {
        let bboxes: Vec<BboxKey> = vec![];
        let result = merge_bboxes(bboxes.into_iter());
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_objects_bbox_with_chars() {
        let chars = vec![
            Char {
                unicode_char: Some("A".to_string()),
                bbox: (
                    OrderedFloat(0.0),
                    OrderedFloat(0.0),
                    OrderedFloat(10.0),
                    OrderedFloat(10.0),
                ),
                rotation_degrees: OrderedFloat(0.0),
                upright: true,
            },
            Char {
                unicode_char: Some("B".to_string()),
                bbox: (
                    OrderedFloat(20.0),
                    OrderedFloat(20.0),
                    OrderedFloat(30.0),
                    OrderedFloat(30.0),
                ),
                rotation_degrees: OrderedFloat(0.0),
                upright: true,
            },
        ];
        let result = get_objects_bbox(&chars);
        assert_eq!(
            result,
            Some((
                OrderedFloat(0.0),
                OrderedFloat(0.0),
                OrderedFloat(30.0),
                OrderedFloat(30.0)
            ))
        );
    }

    #[test]
    fn test_get_objects_bbox_empty() {
        let chars: Vec<Char> = vec![];
        let result = get_objects_bbox(&chars);
        assert_eq!(result, None);
    }
}
