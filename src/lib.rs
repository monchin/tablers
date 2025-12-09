use pdfium_render::prelude::*;
use pyo3::prelude::*;
use std::path::Path;
use std::rc::Rc;

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
    fn open_doc_from_path(
        &self,
        path: &impl AsRef<Path>,
        password: Option<&str>,
    ) -> Result<PdfDocument<'_>, PdfiumError> {
        self.inner.load_pdf_from_file(path, password)
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
    fn py_new(runtime: &PdfiumRuntime, path: String, password: Option<&str>) -> PyResult<Self> {
        let pdfium = runtime.get_inner();

        // use unsafe magic to bypass lifetime checks
        let doc = runtime
            .open_doc_from_path(&path, password.as_deref())
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                    "Failed to open PDF: {:?}",
                    e
                ))
            })?;

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
