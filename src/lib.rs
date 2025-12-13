use crate::edges::{Edge, EdgeType};
use crate::pages::PdfPage;
use pdfium_render::prelude::{PdfDocument, PdfPageIndex, Pdfium, PdfiumError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyString};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

mod chars;
mod clusters;
mod edges;
mod pages;
mod tables;
#[pyclass(unsendable)]
pub struct PdfiumRuntime {
    inner: Rc<Pdfium>,
}
#[pymethods]
impl PdfiumRuntime {
    #[new]
    fn py_new(path: String) -> PyResult<Self> {
        let bindings = Pdfium::bind_to_library(path).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to bind Pdfium: {:?}",
                e
            ))
        })?;
        let pdfium = Pdfium::new(bindings);
        Ok(Self {
            inner: Rc::new(pdfium),
        })
    }
}

impl PdfiumRuntime {
    fn open_doc_from_path<'a>(
        &'a self,
        path: &impl AsRef<Path>,
        password: Option<&'a str>,
    ) -> Result<PdfDocument<'a>, PdfiumError> {
        self.inner.load_pdf_from_file(path, password)
    }

    fn open_doc_from_bytes<'a>(
        &'a self,
        bytes: &'a [u8],
        password: Option<&'a str>,
    ) -> Result<PdfDocument<'a>, PdfiumError> {
        self.inner.load_pdf_from_byte_slice(bytes, password)
    }

    fn get_inner(&self) -> Rc<Pdfium> {
        Rc::clone(&self.inner)
    }
}

// Shared inner state
struct DocumentInner {
    _pdfium: Rc<Pdfium>,
    doc: Option<PdfDocument<'static>>, // None means closed
}

#[pyclass(unsendable)]
pub struct Document {
    inner: Rc<RefCell<DocumentInner>>,
    bottom_origin: bool,
}

#[pymethods]
impl Document {
    #[new]
    #[pyo3(signature=   (runtime, path=None, bytes=None, password=None, bottom_origin = false))]
    fn py_new(
        runtime: &PdfiumRuntime,
        path: Option<String>,
        bytes: Option<&[u8]>,
        password: Option<String>,
        bottom_origin: bool,
    ) -> PyResult<Self> {
        let pdfium = runtime.get_inner();

        let doc = if let Some(path) = path {
            runtime
                .open_doc_from_path(&path, password.as_deref())
                .map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                        "Failed to open PDF: {:?}",
                        e
                    ))
                })?
        } else if let Some(bytes) = bytes {
            runtime
                .open_doc_from_bytes(bytes, password.as_deref())
                .map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                        "Failed to open PDF from bytes: {:?}",
                        e
                    ))
                })?
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Either path or bytes must be provided",
            ));
        };

        let doc_static: PdfDocument<'static> = unsafe { std::mem::transmute(doc) };

        Ok(Self {
            inner: Rc::new(RefCell::new(DocumentInner {
                _pdfium: pdfium,
                doc: Some(doc_static),
            })),
            bottom_origin,
        })
    }

    /// close the document, all the pages would be invalid
    fn close(&self) -> PyResult<()> {
        let mut inner = self.inner.borrow_mut();
        inner.doc = None;
        Ok(())
    }

    fn is_closed(&self) -> bool {
        self.inner.borrow().doc.is_none()
    }

    fn page_count(&self) -> PyResult<usize> {
        let inner = self.inner.borrow();
        let doc = inner.doc.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Document is closed")
        })?;
        Ok(doc.pages().len().into())
    }

    fn get_page(&self, page_num: usize) -> PyResult<Page> {
        let inner = self.inner.borrow();
        let doc = inner.doc.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Document is closed")
        })?;
        let page_count: usize = doc.pages().len().into();
        if page_num >= page_count {
            return Err(PyErr::new::<pyo3::exceptions::PyIndexError, _>(format!(
                "Page index {} out of range (0..{})",
                page_num, page_count
            )));
        }

        Ok(Page {
            doc_inner: Rc::clone(&self.inner),
            inner: PdfPage::new(
                doc.pages().get(page_num as PdfPageIndex).unwrap(),
                self.bottom_origin,
            ),
        })
    }

    fn pages(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let inner = self.inner.borrow();
        let doc = inner.doc.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Document is closed")
        })?;
        let page_count = self.page_count()?;
        let pages: Vec<Page> = (0..page_count)
            .map(|i| Page {
                doc_inner: Rc::clone(&self.inner),
                inner: PdfPage::new(
                    doc.pages().get(i as PdfPageIndex).unwrap(),
                    self.bottom_origin,
                ),
            })
            .collect();

        Ok(PyList::new(py, pages.into_iter().map(|p| p.into_pyobject(py).unwrap()))?.into())
    }
}

#[pyclass(unsendable)]
pub struct Page {
    doc_inner: Rc<RefCell<DocumentInner>>,
    inner: PdfPage,
}

impl Page {
    fn check_valid(&self) -> PyResult<()> {
        if self.doc_inner.borrow().doc.is_none() {
            return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Document is closed",
            ));
        }
        Ok(())
    }
}

#[pymethods]
impl Page {
    #[getter]
    fn width(&self) -> PyResult<f32> {
        self.check_valid()?;
        Ok(self.inner.width())
    }

    #[getter]
    fn height(&self) -> PyResult<f32> {
        self.check_valid()?;
        Ok(self.inner.height())
    }

    fn is_valid(&self) -> bool {
        self.doc_inner.borrow().doc.is_some()
    }

    fn extract_edges(&self) -> PyResult<()> {
        self.check_valid()?;
        self.inner.extract_edges();
        Ok(())
    }

    #[getter]
    fn edges(&self, py: Python<'_>) -> PyResult<Option<Py<PyDict>>> {
        self.check_valid()?;

        let key_namer = |edge_type: &EdgeType| -> &str {
            match edge_type {
                EdgeType::HorizontalLine => "h_line",
                EdgeType::HorizontalRect => "h_rect",
                EdgeType::VerticalLine => "v_line",
                EdgeType::VerticalRect => "v_rect",
            }
        };

        self.inner.extract_edges();

        let edges_ref = self.inner.edges.borrow();

        let edges_map = match edges_ref.as_ref() {
            Some(map) => map,
            None => return Ok(None), // 返回 Python None
        };

        let result = PyDict::new(py);

        for (edge_type, edge_list) in edges_map.iter() {
            let key = key_namer(edge_type);
            let py_list = PyList::new(py, edge_list.clone())?;
            result.set_item(key, py_list)?;
        }

        Ok(Some(result.into())) // 返回 Some(dict)
    }

    fn clear_cache(&self) -> PyResult<()> {
        self.check_valid()?;
        self.inner.clear();
        Ok(())
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
        self.width
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
    fn edge_type(&self) -> &str {
        match self.edge_type {
            EdgeType::HorizontalLine => "h_line",
            EdgeType::HorizontalRect => "h_rect",
            EdgeType::VerticalLine => "v_line",
            EdgeType::VerticalRect => "v_rect",
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Edge(type={}, x1={}, y1={}, x2={}, y2={}, width={}, color=(R{}, G{}, B{}, A{}))",
            self.edge_type(),
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

#[pymodule]
fn tablers(_py: Python<'_>, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<PdfiumRuntime>()?;
    m.add_class::<Document>()?;
    m.add_class::<Page>()?;
    m.add_class::<Edge>()?;
    Ok(())
}
