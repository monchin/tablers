use pdfium_render::prelude::*;
use pyo3::prelude::*;
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

#[pyclass(unsendable)]
pub struct Document {
    _pdfium: Rc<Pdfium>,
    inner: PdfDocument<'static>,
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

        // use unsafe magic to bypass lifetime checks
        let doc_static: PdfDocument<'static> = unsafe { std::mem::transmute(doc) };

        Ok(Self {
            _pdfium: pdfium,
            inner: doc_static,
        })
    }

    fn page_count(&self) -> usize {
        self.inner.pages().len().into()
    }
}

#[pymodule]
fn tablers(_py: Python<'_>, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<PdfiumRuntime>()?;
    m.add_class::<Document>()?;
    Ok(())
}
