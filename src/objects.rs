use ordered_float::OrderedFloat;
use pdfium_render::prelude::PdfColor;
use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
pub struct Objects {
    pub rects: Vec<Rect>,
    pub lines: Vec<Line>,
}

#[pymethods]
impl Objects {
    #[getter]
    fn rects(&self) -> Vec<Rect> {
        self.rects.clone()
    }

    #[getter]
    fn lines(&self) -> Vec<Line> {
        self.lines.clone()
    }
}
pub type Point = (OrderedFloat<f32>, OrderedFloat<f32>);
pub type BboxKey = (OrderedFloat<f32>, OrderedFloat<f32>, OrderedFloat<f32>, OrderedFloat<f32>);
#[pyclass]
#[derive(Clone)]
pub struct Rect{
    pub bbox: BboxKey,
    pub fill_color: PdfColor,
    pub stroke_color: PdfColor,
    #[pyo3(get)]
    pub stroke_width: f32,
}

#[pymethods]
impl Rect {
    #[getter]
    fn bbox(&self) -> (f32, f32, f32, f32) {
        (self.bbox.0.into_inner(), self.bbox.1.into_inner(), self.bbox.2.into_inner(), self.bbox.3.into_inner())
    }

    #[getter]
    fn fill_color(&self) -> (u8, u8, u8, u8) {
        (self.fill_color.red(), self.fill_color.green(), self.fill_color.blue(), self.fill_color.alpha())
    }

    #[getter]
    fn stroke_color(&self) -> (u8, u8, u8, u8) {
        (self.stroke_color.red(), self.stroke_color.green(), self.stroke_color.blue(), self.stroke_color.alpha())
    }
}

#[pyclass]
#[derive(Clone)]
pub struct Line {
    pub line_type: LineType,
    pub points: Vec<Point>,
    pub color: PdfColor,
    #[pyo3(get)]
    pub width: f32,
}

#[pymethods]
impl Line {
    #[getter]
    fn line_type(&self) -> &str {
        match self.line_type {
            LineType::Straight => "straight",
            LineType::Curve => "curve",
        }
    }

    #[getter]
    fn points(&self) -> Vec<(f32, f32)> {
        self.points.iter().map(|p| (p.0.into_inner(), p.1.into_inner())).collect()
    }

    #[getter]
    fn color(&self) -> (u8, u8, u8, u8) {
        (self.color.red(), self.color.green(), self.color.blue(), self.color.alpha())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum Orientation {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum LineType {
    Straight,
    Curve,
}

pub(crate) fn is_rect(points: &[Point]) -> bool {
    if !points.len() == 5 || points[0] != points[4] { return false; }
    if points[0].0 == points[1].0 && points[1].1 == points[2].1 && points[2].0 == points[3].0 && points[3].1 == points[0].1 {
        return true;
    }
    if points[0].1 == points[1].1 && points[1].0 == points[2].0 && points[2].1 == points[3].1 && points[3].0 == points[0].0 {
        return true;
    }
    false
}


