use crate::edges::Edge;
use crate::objects::*;
use crate::pages::Page;
use crate::tables::{Table, TableCell, TableFinder, TfSettings, find_tables};
use pdfium_render::prelude::{PdfDocument, PdfPageIndex, Pdfium, PdfiumError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
mod clusters;
mod edges;
mod objects;
mod pages;
mod tables;
mod words;

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
}

#[pymethods]
impl Document {
    #[new]
    #[pyo3(signature=   (runtime, path=None, bytes=None, password=None))]
    fn py_new(
        runtime: &PdfiumRuntime,
        path: Option<String>,
        bytes: Option<&[u8]>,
        password: Option<String>,
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

    fn get_page(&self, page_idx: usize) -> PyResult<PyPage> {
        let inner = self.inner.borrow();
        let doc = inner.doc.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Document is closed")
        })?;
        let page_count: usize = doc.pages().len().into();
        if page_idx >= page_count {
            return Err(PyErr::new::<pyo3::exceptions::PyIndexError, _>(format!(
                "Page index {} out of range (0..{})",
                page_idx, page_count
            )));
        }
        Ok(PyPage {
            doc_inner: Rc::clone(&self.inner),
            inner: Page::new(doc.pages().get(page_idx as PdfPageIndex).unwrap(), page_idx),
        })
    }

    fn pages(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let inner = self.inner.borrow();
        let doc = inner.doc.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Document is closed")
        })?;
        let page_count = self.page_count()?;
        let pages: Vec<PyPage> = (0..page_count)
            .map(|i| PyPage {
                doc_inner: Rc::clone(&self.inner),
                inner: Page::new(doc.pages().get(i as PdfPageIndex).unwrap(), i),
            })
            .collect();

        Ok(PyList::new(py, pages.into_iter().map(|p| p.into_pyobject(py).unwrap()))?.into())
    }
}

#[pyclass(unsendable, name = "Page")]
pub struct PyPage {
    doc_inner: Rc<RefCell<DocumentInner>>,
    inner: Page,
}

impl PyPage {
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
impl PyPage {
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

    #[getter]
    fn rotation_degrees(&self) -> PyResult<f32> {
        self.check_valid()?;
        Ok(self.inner.rotation_degrees().as_degrees())
    }

    fn is_valid(&self) -> bool {
        self.doc_inner.borrow().doc.is_some()
    }

    fn extract_objects(&self) -> PyResult<()> {
        self.check_valid()?;
        self.inner.extract_objects();
        Ok(())
    }

    #[getter]
    fn objects(&self) -> PyResult<Option<Objects>> {
        self.check_valid()?;
        if self.inner.objects.borrow().is_none() {
            return Ok(None);
        }
        Ok(self.inner.objects.borrow().clone())
    }

    // #[getter]
    // fn most_chars_rotation_degrees(&self) -> PyResult<f32> {
    //     self.check_valid()?;
    //     Ok(self.inner.most_chars_rotation_degrees.borrow().clone())
    // }

    fn clear_cache(&self) -> PyResult<()> {
        self.check_valid()?;
        self.inner.clear();
        Ok(())
    }
}

#[pyfunction]
pub fn get_edges(page: &PyPage, settings: Option<&Bound<'_, PyDict>>) -> PyResult<Py<PyDict>> {
    page.check_valid()?;
    let settings = Rc::new(TfSettings::py_new(settings));
    let edges = TableFinder::new(settings).get_edges(&page.inner);

    Python::attach(|py| {
        let res = PyDict::new(py);
        let horizontal_edges: Vec<Edge> = edges
            .get(&Orientation::Horizontal)
            .map(|edges| edges.clone())
            .unwrap_or_default();
        res.set_item("h", horizontal_edges)?;
        let vertical_edges: Vec<Edge> = edges
            .get(&Orientation::Vertical)
            .map(|edges| edges.clone())
            .unwrap_or_default();
        res.set_item("v", vertical_edges)?;
        Ok(res.unbind())
    })
}

#[pyfunction]
#[pyo3(name = "find_tables")]
#[pyo3(signature = (page, extract_text, **kwargs))]
fn py_find_tables(
    page: &PyPage,
    extract_text: bool,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<(Vec<(f32, f32, f32, f32)>, Vec<Table>)> {
    let settings = Rc::new(TfSettings::py_new(kwargs));
    let (cell_bboxes, tables) = find_tables(&page.inner, settings.clone(), extract_text);
    let cell_bboxes = cell_bboxes
        .into_iter()
        .map(|bbox| {
            (
                bbox.0.into_inner(),
                bbox.1.into_inner(),
                bbox.2.into_inner(),
                bbox.3.into_inner(),
            )
        })
        .collect();
    Ok((cell_bboxes, tables))
}

#[pymodule]
fn tablers(_py: Python<'_>, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<PdfiumRuntime>()?;
    m.add_class::<Document>()?;
    m.add_class::<PyPage>()?;
    m.add_class::<Edge>()?;
    m.add_class::<TableCell>()?;
    m.add_class::<Table>()?;
    m.add_class::<TfSettings>()?;
    m.add_function(pyo3::wrap_pyfunction!(py_find_tables, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(get_edges, m)?)?;
    Ok(())
}
