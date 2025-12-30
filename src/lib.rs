use crate::edges::Edge;
use crate::objects::*;
use crate::pages::Page;
use crate::settings::*;
use crate::tables::*;
use ordered_float::OrderedFloat;
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
mod settings;
mod tables;
#[cfg(test)]
mod test_utils;
mod words;

type PyBbox = (f32, f32, f32, f32);

/// A wrapper around the Pdfium library runtime.
///
/// This struct holds the Pdfium instance and provides methods to interact with PDF documents.
/// It is unsendable because the underlying Pdfium library is not thread-safe.
#[pyclass(unsendable)]
pub struct PdfiumRuntime {
    inner: Rc<Pdfium>,
}
#[pymethods]
impl PdfiumRuntime {
    /// Creates a new PdfiumRuntime instance by loading the Pdfium library from the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path to the Pdfium dynamic library.
    ///
    /// # Returns
    ///
    /// A new `PdfiumRuntime` instance or a Python error if the library fails to load.
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
    /// Opens a PDF document from a file path.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path to the PDF document.
    /// * `password` - Optional password for encrypted PDFs.
    ///
    /// # Returns
    ///
    /// A `PdfDocument` instance or a `PdfiumError` if the file cannot be opened.
    fn open_doc_from_path<'a>(
        &'a self,
        path: &impl AsRef<Path>,
        password: Option<&'a str>,
    ) -> Result<PdfDocument<'a>, PdfiumError> {
        self.inner.load_pdf_from_file(path, password)
    }

    /// Opens a PDF document from a byte slice.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The PDF document content as bytes.
    /// * `password` - Optional password for encrypted PDFs.
    ///
    /// # Returns
    ///
    /// A `PdfDocument` instance or a `PdfiumError` if the bytes cannot be parsed.
    fn open_doc_from_bytes<'a>(
        &'a self,
        bytes: &'a [u8],
        password: Option<&'a str>,
    ) -> Result<PdfDocument<'a>, PdfiumError> {
        self.inner.load_pdf_from_byte_vec(bytes.to_vec(), password)
    }

    /// Returns a reference-counted pointer to the inner Pdfium instance.
    fn get_inner(&self) -> Rc<Pdfium> {
        Rc::clone(&self.inner)
    }

    /// Creates a new PdfiumRuntime from an existing Pdfium instance (for testing).
    #[cfg(test)]
    fn from_pdfium(pdfium: &Pdfium) -> Self {
        Self {
            inner: Rc::new(pdfium.clone()),
        }
    }
}

/// Shared inner state for the Document.
///
/// Contains the Pdfium reference and the actual PDF document.
/// The document is wrapped in an Option to support closing.
struct DocumentInner {
    _pdfium: Rc<Pdfium>,
    doc: Option<PdfDocument<'static>>, // None means closed
}

/// Represents an opened PDF document.
///
/// This struct provides methods to access pages and metadata of a PDF document.
/// The document can be closed explicitly, after which all operations will fail.
#[pyclass(unsendable)]
pub struct Document {
    inner: Rc<RefCell<DocumentInner>>,
}

#[pymethods]
impl Document {
    /// Creates a new Document instance from a file path or bytes.
    ///
    /// # Arguments
    ///
    /// * `runtime` - The PdfiumRuntime instance to use.
    /// * `path` - Optional file path to the PDF document.
    /// * `bytes` - Optional PDF content as bytes.
    /// * `password` - Optional password for encrypted PDFs.
    ///
    /// # Returns
    ///
    /// A new `Document` instance or a Python error if the document cannot be opened.
    ///
    /// # Note
    ///
    /// Either `path` or `bytes` must be provided, but not both.
    #[new]
    #[pyo3(signature=(runtime, path=None, bytes=None, password=None))]
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

    /// Checks if the document has been closed.
    ///
    /// # Returns
    ///
    /// `true` if the document is closed, `false` otherwise.
    fn is_closed(&self) -> bool {
        self.inner.borrow().doc.is_none()
    }

    /// Returns the total number of pages in the document.
    ///
    /// # Returns
    ///
    /// The page count or a Python error if the document is closed.
    fn page_count(&self) -> PyResult<usize> {
        let inner = self.inner.borrow();
        let doc = inner.doc.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Document is closed")
        })?;
        let count: i32 = doc.pages().len();
        if count < 0 {
            return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Invalid page count",
            ));
        }
        Ok(count as usize)
    }

    /// Retrieves a specific page from the document by index.
    ///
    /// # Arguments
    ///
    /// * `page_idx` - The zero-based index of the page to retrieve.
    ///
    /// # Returns
    ///
    /// A `PyPage` instance or a Python error if the index is out of range or document is closed.
    fn get_page(&self, page_idx: usize) -> PyResult<PyPage> {
        let inner = self.inner.borrow();
        let doc = inner.doc.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Document is closed")
        })?;
        let count: i32 = doc.pages().len();
        if count < 0 {
            return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Invalid page count",
            ));
        }
        let page_count: usize = count as usize;
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

    /// Returns an iterator over pages (memory efficient for large PDFs)
    fn pages(&self) -> PyResult<PyPageIterator> {
        self.__iter__()
    }

    /// Returns an iterator over all pages in the document.
    ///
    /// # Returns
    ///
    /// A `PyPageIterator` or a Python error if the document is closed.
    fn __iter__(&self) -> PyResult<PyPageIterator> {
        // Check if document is valid
        let inner = self.inner.borrow();
        if inner.doc.is_none() {
            return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Document is closed",
            ));
        }
        drop(inner);

        let page_count = self.page_count()?;
        Ok(PyPageIterator {
            doc_inner: Rc::clone(&self.inner),
            current_idx: 0,
            page_count,
        })
    }

    /// Context manager entry point.
    ///
    /// # Returns
    ///
    /// A reference to self for use in `with` statements.
    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    /// Context manager exit point.
    ///
    /// Closes the document when exiting the `with` block.
    ///
    /// # Arguments
    ///
    /// * `_exc_type` - The exception type (if any).
    /// * `_exc_val` - The exception value (if any).
    /// * `_exc_tb` - The exception traceback (if any).
    ///
    /// # Returns
    ///
    /// `false` to indicate that exceptions should not be suppressed.
    fn __exit__(
        &self,
        _exc_type: Option<&Bound<'_, PyAny>>,
        _exc_val: Option<&Bound<'_, PyAny>>,
        _exc_tb: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<bool> {
        self.close()?;
        Ok(false)
    }
}

/// Iterator for traversing pages in a PDF document.
///
/// This iterator is memory-efficient for large PDFs as it loads pages on demand.
#[pyclass(unsendable, name = "PageIterator")]
pub struct PyPageIterator {
    doc_inner: Rc<RefCell<DocumentInner>>,
    current_idx: usize,
    page_count: usize,
}

#[pymethods]
impl PyPageIterator {
    /// Returns self as the iterator.
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    /// Returns the next page in the iteration.
    ///
    /// # Returns
    ///
    /// The next `PyPage` or `None` if iteration is complete.
    fn __next__(&mut self) -> PyResult<Option<PyPage>> {
        if self.current_idx >= self.page_count {
            return Ok(None);
        }

        let inner = self.doc_inner.borrow();
        let doc = inner.doc.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Document is closed")
        })?;

        let page_idx = self.current_idx;
        self.current_idx += 1;

        Ok(Some(PyPage {
            doc_inner: Rc::clone(&self.doc_inner),
            inner: Page::new(doc.pages().get(page_idx as PdfPageIndex).unwrap(), page_idx),
        }))
    }
}

/// Represents a single page in a PDF document.
///
/// Provides access to page properties like dimensions and rotation,
/// as well as methods to extract objects and text from the page.
#[pyclass(unsendable, name = "Page")]
pub struct PyPage {
    doc_inner: Rc<RefCell<DocumentInner>>,
    inner: Page,
}

impl PyPage {
    /// Checks if the parent document is still valid (not closed).
    ///
    /// # Returns
    ///
    /// `Ok(())` if valid, or a Python error if the document has been closed.
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
    /// Returns the index of the page within the document.
    #[getter]
    fn page_idx(&self) -> PyResult<usize> {
        self.check_valid()?;
        Ok(self.inner.page_idx)
    }

    /// Returns the width of the page in points.
    #[getter]
    fn width(&self) -> PyResult<f32> {
        self.check_valid()?;
        Ok(self.inner.width())
    }

    /// Returns the height of the page in points.
    #[getter]
    fn height(&self) -> PyResult<f32> {
        self.check_valid()?;
        Ok(self.inner.height())
    }

    /// Returns the rotation of the page in degrees.
    #[getter]
    fn rotation_degrees(&self) -> PyResult<f32> {
        self.check_valid()?;
        Ok(self.inner.rotation_degrees().as_degrees())
    }

    /// Checks if the page reference is still valid (document not closed).
    ///
    /// # Returns
    ///
    /// `true` if the page is valid, `false` otherwise.
    fn is_valid(&self) -> bool {
        self.doc_inner.borrow().doc.is_some()
    }

    /// Extracts all objects (characters, lines, rectangles) from the page.
    ///
    /// This method caches the extracted objects for subsequent access.
    fn extract_objects(&self) -> PyResult<()> {
        self.check_valid()?;
        self.inner.extract_objects();
        Ok(())
    }

    /// Returns the extracted objects from the page.
    ///
    /// # Returns
    ///
    /// An `Objects` instance containing all extracted objects, or `None` if not yet extracted.
    #[getter]
    fn objects(&self) -> PyResult<Option<Objects>> {
        self.check_valid()?;
        if self.inner.objects.borrow().is_none() {
            return Ok(None);
        }
        Ok(self.inner.objects.borrow().clone())
    }

    /// Clears the cached objects to free memory.
    fn clear_cache(&self) -> PyResult<()> {
        self.check_valid()?;
        self.inner.clear();
        Ok(())
    }
}

/// Extracts edges (lines and rectangle borders) from a PDF page.
///
/// # Arguments
///
/// * `page` - The PDF page to extract edges from.
/// * `settings` - Optional dictionary of settings for edge extraction.
///
/// # Returns
///
/// A dictionary with keys "h" (horizontal edges) and "v" (vertical edges).
#[pyfunction]
pub fn get_edges(page: &PyPage, settings: Option<&Bound<'_, PyDict>>) -> PyResult<Py<PyDict>> {
    page.check_valid()?;
    let settings = Rc::new(TfSettings::py_new(settings)?);
    let edges = TableFinder::new(settings).get_edges(&page.inner);

    Python::attach(|py| {
        let res = PyDict::new(py);
        let horizontal_edges: Vec<Edge> = edges
            .get(&Orientation::Horizontal)
            .cloned()
            .unwrap_or_default();
        res.set_item("h", horizontal_edges)?;
        let vertical_edges: Vec<Edge> = edges
            .get(&Orientation::Vertical)
            .cloned()
            .unwrap_or_default();
        res.set_item("v", vertical_edges)?;
        Ok(res.unbind())
    })
}

/// Converts a Rust bounding box to a Python tuple.
///
/// # Arguments
///
/// * `bbox` - The Rust bounding box (x1, y1, x2, y2) with OrderedFloat values.
///
/// # Returns
///
/// A tuple of f32 values representing the bounding box.
fn rs_bbox_to_py_bbox(bbox: &BboxKey) -> PyBbox {
    (
        bbox.0.into_inner(),
        bbox.1.into_inner(),
        bbox.2.into_inner(),
        bbox.3.into_inner(),
    )
}

/// Converts a Python bounding box tuple to a Rust BboxKey.
///
/// # Arguments
///
/// * `bbox` - The Python bounding box tuple (x1, y1, x2, y2).
///
/// # Returns
///
/// A BboxKey with OrderedFloat values.
fn py_bbox_to_rs_bbox(bbox: &PyBbox) -> BboxKey {
    (
        OrderedFloat(bbox.0),
        OrderedFloat(bbox.1),
        OrderedFloat(bbox.2),
        OrderedFloat(bbox.3),
    )
}
/// Finds all table cell bounding boxes in a PDF page.
///
/// # Arguments
///
/// * `page` - The PDF page to analyze.
/// * `tf_settings` - Optional TableFinder settings object.
/// * `kwargs` - Optional keyword arguments for settings.
///
/// # Returns
///
/// A list of bounding boxes (x1, y1, x2, y2) for each detected cell.
#[pyfunction]
#[pyo3(name="find_all_cells_bboxes", signature = (page, tf_settings=None, **kwargs))]
fn py_find_all_cells_bboxes(
    page: &PyPage,
    tf_settings: Option<TfSettings>,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<Vec<PyBbox>> {
    let settings;
    if let Some(tf_settings) = tf_settings {
        settings = Rc::new(tf_settings);
    } else {
        settings = Rc::new(TfSettings::py_new(kwargs)?);
    };
    let cells = find_all_cells_bboxes(&page.inner, settings.clone());
    Ok(cells.iter().map(rs_bbox_to_py_bbox).collect())
}

/// Constructs tables from a list of cell bounding boxes.
///
/// # Arguments
///
/// * `cells` - A list of cell bounding boxes.
/// * `extract_text` - Whether to extract text content from cells.
/// * `pdf_page` - The PDF page (required if extract_text is true).
/// * `we_settings` - Optional word extraction settings.
/// * `kwargs` - Optional keyword arguments for settings.
///
/// # Returns
///
/// A list of Table objects constructed from the cells.
#[pyfunction]
#[pyo3(name = "find_tables_from_cells", signature = (cells,extract_text, pdf_page=None, we_settings=None, **kwargs))]
fn py_find_tables_from_cells(
    cells: &Bound<'_, PyList>,
    extract_text: bool,
    pdf_page: Option<&PyPage>,
    we_settings: Option<WordsExtractSettings>,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<Vec<Table>> {
    let cells: Vec<BboxKey> = cells
        .iter()
        .map(|item| {
            let bbox: PyBbox = item.extract()?;
            Ok(py_bbox_to_rs_bbox(&bbox))
        })
        .collect::<PyResult<Vec<_>>>()?;
    let settings_value = if extract_text {
        Some(match we_settings {
            Some(s) => s,
            None => WordsExtractSettings::py_new(kwargs)?,
        })
    } else {
        None
    };
    let settings = settings_value.as_ref();

    let page = match extract_text {
        true => match pdf_page {
            Some(pdf_page) => Some(&pdf_page.inner),
            None => {
                return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                    "pdf_page is required when extract_text is true",
                ));
            }
        },
        false => None,
    };

    let tables = find_tables_from_cells(&cells, extract_text, page, settings);
    Ok(tables)
}
/// Finds all tables in a PDF page.
///
/// # Arguments
///
/// * `page` - The PDF page to analyze.
/// * `extract_text` - Whether to extract text content from table cells.
/// * `tf_settings` - Optional TableFinder settings object.
/// * `kwargs` - Optional keyword arguments for settings.
///
/// # Returns
///
/// A list of Table objects found in the page.
#[pyfunction]
#[pyo3(name = "find_tables", signature = (page, extract_text, tf_settings=None, **kwargs))]
fn py_find_tables(
    page: &PyPage,
    extract_text: bool,
    tf_settings: Option<TfSettings>,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<Vec<Table>> {
    let settings;
    if let Some(tf_settings) = tf_settings {
        settings = Rc::new(tf_settings);
    } else {
        settings = Rc::new(TfSettings::py_new(kwargs)?);
    };
    let tables = find_tables(&page.inner, settings.clone(), extract_text);
    Ok(tables)
}

/// Initializes the tablers Python module.
///
/// This function is called by Python when importing the module and registers
/// all classes and functions available to Python.
#[pymodule]
fn tablers(_py: Python<'_>, m: &Bound<PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<PdfiumRuntime>()?;
    m.add_class::<Document>()?;
    m.add_class::<PyPage>()?;
    m.add_class::<PyPageIterator>()?;
    m.add_class::<Edge>()?;
    m.add_class::<TableCell>()?;
    m.add_class::<Table>()?;
    m.add_class::<TfSettings>()?;
    m.add_class::<WordsExtractSettings>()?;
    m.add_function(pyo3::wrap_pyfunction!(py_find_all_cells_bboxes, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(py_find_tables_from_cells, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(py_find_tables, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(get_edges, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::load_pdfium;

    #[test]
    fn test_open_encrypted_pdf_from_path_with_password() {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let pdfium = load_pdfium();
        let runtime = PdfiumRuntime::from_pdfium(pdfium);

        let pdf_path = format!(
            "{}/tests/data/test-encryption-pswd-qwerty.pdf",
            project_root
        );
        let doc = runtime.open_doc_from_path(&pdf_path, Some("qwerty"));

        assert!(
            doc.is_ok(),
            "Should open encrypted PDF with correct password"
        );
        let doc = doc.unwrap();
        assert!(doc.pages().len() > 0, "Document should have pages");
    }

    #[test]
    fn test_open_encrypted_pdf_from_path_without_password_fails() {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let pdfium = load_pdfium();
        let runtime = PdfiumRuntime::from_pdfium(pdfium);

        let pdf_path = format!(
            "{}/tests/data/test-encryption-pswd-qwerty.pdf",
            project_root
        );
        let doc = runtime.open_doc_from_path(&pdf_path, None);

        assert!(
            doc.is_err(),
            "Should fail to open encrypted PDF without password"
        );
    }

    #[test]
    fn test_open_encrypted_pdf_from_path_with_wrong_password_fails() {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let pdfium = load_pdfium();
        let runtime = PdfiumRuntime::from_pdfium(pdfium);

        let pdf_path = format!(
            "{}/tests/data/test-encryption-pswd-qwerty.pdf",
            project_root
        );
        let doc = runtime.open_doc_from_path(&pdf_path, Some("wrong_password"));

        assert!(
            doc.is_err(),
            "Should fail to open encrypted PDF with wrong password"
        );
    }

    #[test]
    fn test_open_encrypted_pdf_from_bytes_with_password() {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let pdfium = load_pdfium();
        let runtime = PdfiumRuntime::from_pdfium(pdfium);

        let pdf_path = format!(
            "{}/tests/data/test-encryption-pswd-qwerty.pdf",
            project_root
        );
        let bytes = std::fs::read(&pdf_path).unwrap();
        let doc = runtime.open_doc_from_bytes(&bytes, Some("qwerty"));

        assert!(
            doc.is_ok(),
            "Should open encrypted PDF from bytes with correct password"
        );
        let doc = doc.unwrap();
        assert!(doc.pages().len() > 0, "Document should have pages");
    }

    #[test]
    fn test_open_encrypted_pdf_from_bytes_without_password_fails() {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let pdfium = load_pdfium();
        let runtime = PdfiumRuntime::from_pdfium(pdfium);

        let pdf_path = format!(
            "{}/tests/data/test-encryption-pswd-qwerty.pdf",
            project_root
        );
        let bytes = std::fs::read(&pdf_path).unwrap();
        let doc = runtime.open_doc_from_bytes(&bytes, None);

        assert!(
            doc.is_err(),
            "Should fail to open encrypted PDF from bytes without password"
        );
    }

    #[test]
    fn test_open_encrypted_pdf_from_bytes_with_wrong_password_fails() {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let pdfium = load_pdfium();
        let runtime = PdfiumRuntime::from_pdfium(pdfium);

        let pdf_path = format!(
            "{}/tests/data/test-encryption-pswd-qwerty.pdf",
            project_root
        );
        let bytes = std::fs::read(&pdf_path).unwrap();
        let doc = runtime.open_doc_from_bytes(&bytes, Some("wrong_password"));

        assert!(
            doc.is_err(),
            "Should fail to open encrypted PDF from bytes with wrong password"
        );
    }
}
