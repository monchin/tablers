use pdfium_render::prelude::*;
use pyo3::prelude::*;
use pyo3::types::PyList;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

mod clusters;
mod edges;

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
}

#[pymethods]
impl Document {
    #[new]
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

    fn get_page(&self, page_num: usize) -> PyResult<Page> {
        // 先检查文档是否打开，以及页码是否有效
        {
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
        }

        Ok(Page {
            doc_inner: Rc::clone(&self.inner),
            page_index: page_num,
        })
    }

    /// 获取所有页面
    fn pages(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let page_count = self.page_count()?;
        let pages: Vec<Page> = (0..page_count)
            .map(|i| Page {
                doc_inner: Rc::clone(&self.inner),
                page_index: i,
            })
            .collect();

        Ok(PyList::new(py, pages.into_iter().map(|p| p.into_pyobject(py).unwrap()))?.into())
    }
}

#[pyclass(unsendable)]
pub struct Page {
    doc_inner: Rc<RefCell<DocumentInner>>,
    page_index: usize,
}

impl Page {
    fn with_page<T, F>(&self, f: F) -> PyResult<T>
    where
        F: FnOnce(&PdfPage) -> T,
    {
        let inner = self.doc_inner.borrow();
        let doc = inner.doc.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Document is closed")
        })?;
        let page = doc
            .pages()
            .get(self.page_index.try_into().unwrap())
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                    "Failed to get page: {:?}",
                    e
                ))
            })?;
        Ok(f(&page))
    }
}

#[pymethods]
impl Page {
    #[getter]
    fn width(&self) -> PyResult<f32> {
        self.with_page(|page| page.width().value)
    }

    #[getter]
    fn height(&self) -> PyResult<f32> {
        self.with_page(|page| page.height().value)
    }

    #[getter]
    fn page_index(&self) -> usize {
        self.page_index
    }

    /// 检查页面是否有效
    fn is_valid(&self) -> bool {
        self.doc_inner.borrow().doc.is_some()
    }
}

#[pymodule]
fn tablers(_py: Python<'_>, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<PdfiumRuntime>()?;
    m.add_class::<Document>()?;
    m.add_class::<Page>()?;
    Ok(())
}
