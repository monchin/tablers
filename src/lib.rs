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
use std::sync::OnceLock;

/// Global storage for the Pdfium instance.
/// This ensures that `bind_to_library` is only called once per process.
static PDFIUM: OnceLock<Pdfium> = OnceLock::new();

/// Gets a reference to the global Pdfium instance, initializing it if necessary.
/// This is used internally by test_utils to share the same Pdfium instance.
#[cfg(test)]
pub(crate) fn get_or_init_pdfium() -> &'static Pdfium {
    PDFIUM.get_or_init(|| {
        let project_root = env!("CARGO_MANIFEST_DIR");

        #[cfg(target_os = "windows")]
        let pdfium_path = format!("{}/python/tablers/pdfium.dll", project_root);
        #[cfg(target_os = "macos")]
        let pdfium_path = format!("{}/python/tablers/libpdfium.dylib", project_root);
        #[cfg(target_os = "linux")]
        let pdfium_path = format!("{}/python/tablers/libpdfium.so.1", project_root);

        let bindings =
            Pdfium::bind_to_library(&pdfium_path).expect("Failed to bind Pdfium library");
        Pdfium::new(bindings)
    })
}
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
    /// If the library has already been initialized, the existing instance is reused
    /// and the provided path is ignored.
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
        // If already initialized, reuse the existing instance
        if let Some(pdfium) = PDFIUM.get() {
            return Ok(Self {
                inner: Rc::new(pdfium.clone()),
            });
        }

        // First initialization
        let bindings = Pdfium::bind_to_library(&path).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to bind Pdfium: {:?}",
                e
            ))
        })?;
        let pdfium = Pdfium::new(bindings);

        // Try to set the global instance (may fail if another thread set it first)
        let _ = PDFIUM.set(pdfium);

        // Return the global instance (either ours or the one set by another thread)
        Ok(Self {
            inner: Rc::new(PDFIUM.get().unwrap().clone()),
        })
    }

    /// Returns whether the Pdfium library has been initialized.
    #[staticmethod]
    #[pyo3(name = "is_initialized")]
    fn py_is_initialized() -> bool {
        PDFIUM.get().is_some()
    }
}

impl PdfiumRuntime {
    /// Creates a new PdfiumRuntime by initializing the Pdfium library from the specified path.
    ///
    /// If the library has already been initialized, the existing instance is reused
    /// and the provided path is ignored.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path to the Pdfium dynamic library.
    ///
    /// # Returns
    ///
    /// A new `PdfiumRuntime` instance or a `PdfiumError` if the library fails to load.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let runtime = PdfiumRuntime::new("path/to/pdfium.dll")?;
    /// ```
    pub fn new(path: impl AsRef<Path>) -> Result<Self, PdfiumError> {
        // If already initialized, reuse the existing instance
        if let Some(pdfium) = PDFIUM.get() {
            return Ok(Self {
                inner: Rc::new(pdfium.clone()),
            });
        }

        // First initialization
        let bindings = Pdfium::bind_to_library(path.as_ref())?;
        let pdfium = Pdfium::new(bindings);

        // Try to set the global instance (may fail if another thread set it first)
        let _ = PDFIUM.set(pdfium);

        // Return the global instance (either ours or the one set by another thread)
        Ok(Self {
            inner: Rc::new(PDFIUM.get().unwrap().clone()),
        })
    }

    /// Gets an existing PdfiumRuntime if the library has already been initialized.
    ///
    /// # Returns
    ///
    /// `Some(PdfiumRuntime)` if the library has been initialized, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(runtime) = PdfiumRuntime::get() {
    ///     // Use existing runtime
    /// } else {
    ///     // Initialize with PdfiumRuntime::new()
    /// }
    /// ```
    pub fn get() -> Option<Self> {
        PDFIUM.get().map(|pdfium| Self {
            inner: Rc::new(pdfium.clone()),
        })
    }

    /// Returns whether the Pdfium library has been initialized.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if PdfiumRuntime::is_initialized() {
    ///     let runtime = PdfiumRuntime::get().unwrap();
    /// }
    /// ```
    pub fn is_initialized() -> bool {
        PDFIUM.get().is_some()
    }

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

/// Creates a new, unencrypted copy of a PDF document as a byte buffer.
///
/// All pages from `doc` are imported into a fresh `PdfDocument` created via
/// `pdfium.create_new_pdf()`.  Because the new document carries no security
/// settings, the returned bytes can be opened without a password even when the
/// source `doc` was encrypted.
fn save_doc_without_security(
    pdfium: &Pdfium,
    doc: &PdfDocument<'_>,
) -> Result<Vec<u8>, pdfium_render::prelude::PdfiumError> {
    use pdfium_render::prelude::PdfPageIndex;

    let mut new_doc = pdfium.create_new_pdf()?;
    let page_count = doc.pages().len();
    if page_count > 0 {
        new_doc.pages_mut().copy_page_range_from_document(
            doc,
            0..=((page_count - 1) as PdfPageIndex),
            0,
        )?;
    }
    new_doc.save_to_bytes()
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
pub struct Pyo3Doc {
    inner: Rc<RefCell<DocumentInner>>,
}

#[pymethods]
impl Pyo3Doc {
    /// Creates a new Pyo3Doc instance from a file path or bytes.
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
    /// A new `Pyo3Doc` instance or a Python error if the document cannot be opened.
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

    /// Serialize the document to bytes, **always without encryption**.
    ///
    /// Internally this creates a brand-new, empty PDF document, copies every page
    /// from the current document into it, and serializes the result via
    /// `FPDF_SaveAsCopy`.  Because the destination document carries no security
    /// settings, the returned bytes can always be opened without a password—even
    /// when the source was an encrypted PDF that was unlocked with a password.
    ///
    /// # Warning
    ///
    /// If the original document was password-protected, calling this method
    /// effectively **strips the encryption**.  The caller is responsible for
    /// ensuring this is intentional and appropriate for their use case.
    ///
    /// # Performance
    ///
    /// This method is **not** cheap: it allocates and populates a new in-memory
    /// PDF document on every call.  For large documents the peak memory usage
    /// will temporarily reach ~2× the document size.  Do not call this in a
    /// tight loop or on every request; cache the result if you need it more than once.
    ///
    /// # Returns
    ///
    /// The serialized PDF bytes, or a Python error if the document is closed or
    /// serialization fails.
    fn save_to_bytes(&self) -> PyResult<Vec<u8>> {
        let inner = self.inner.borrow();
        let doc = inner.doc.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Document is closed")
        })?;
        let pdfium: &Pdfium = &inner._pdfium;
        save_doc_without_security(pdfium, doc).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to serialize PDF without security: {:?}",
                e
            ))
        })
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
    /// A `Pyo3Page` instance or a Python error if the index is out of range or document is closed.
    fn get_page(&self, page_idx: usize) -> PyResult<Pyo3Page> {
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
        Ok(Pyo3Page {
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
    /// The next `Pyo3Page` or `None` if iteration is complete.
    fn __next__(&mut self) -> PyResult<Option<Pyo3Page>> {
        if self.current_idx >= self.page_count {
            return Ok(None);
        }

        let inner = self.doc_inner.borrow();
        let doc = inner.doc.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Document is closed")
        })?;

        let page_idx = self.current_idx;
        self.current_idx += 1;

        Ok(Some(Pyo3Page {
            doc_inner: Rc::clone(&self.doc_inner),
            inner: Page::new(doc.pages().get(page_idx as PdfPageIndex).unwrap(), page_idx),
        }))
    }
}

/// Represents a single page in a PDF document.
///
/// Provides access to page properties like dimensions and rotation,
/// as well as methods to extract objects and text from the page.
#[pyclass(unsendable, name = "Pyo3Page")]
pub struct Pyo3Page {
    doc_inner: Rc<RefCell<DocumentInner>>,
    inner: Page,
}

impl Pyo3Page {
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
impl Pyo3Page {
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

/// Extracts edges (lines and rectangle borders) from a PDF page or from explicit edges.
///
/// # Arguments
///
/// * `page` - The PDF page to extract edges from. Can be None only if both
///   horizontal_strategy and vertical_strategy are set to "explicit".
/// * `tf_settings` - Optional TfSettings object for edge extraction.
/// * `kwargs` - Optional keyword arguments for settings.
///
/// # Returns
///
/// A dictionary with keys "h" (horizontal edges) and "v" (vertical edges).
///
/// # Raises
///
/// RuntimeError: If page is None and either strategy is not "explicit".
#[pyfunction]
#[pyo3(name = "get_edges", signature = (page=None, tf_settings=None, **kwargs))]
fn py_get_edges(
    page: Option<&Pyo3Page>,
    tf_settings: Option<TfSettings>,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyDict>> {
    if let Some(p) = page {
        p.check_valid()?;
    }

    let settings = if let Some(s) = tf_settings {
        Rc::new(s)
    } else {
        Rc::new(TfSettings::py_new(kwargs)?)
    };

    // Validate that page can only be None when both strategies are explicit
    if page.is_none()
        && (settings.horizontal_strategy != StrategyType::Explicit
            || settings.vertical_strategy != StrategyType::Explicit)
    {
        return Err(pyo3::exceptions::PyRuntimeError::new_err(
            "page can only be None when both horizontal_strategy and vertical_strategy are 'explicit'",
        ));
    }

    let page_ref = page.map(|p| &p.inner);
    let edges = TableFinder::new(settings).get_edges(page_ref);

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

/// Computes intersection points from a set of horizontal and vertical edges.
///
/// # Arguments
///
/// * `h_edges` - A list of horizontal edges (as returned by ``get_edges``).
/// * `v_edges` - A list of vertical edges (as returned by ``get_edges``).
/// * `tf_settings` - Optional TfSettings object for tolerance configuration.
/// * `kwargs` - Optional keyword arguments for settings.
///
/// # Returns
///
/// A dictionary mapping ``(x, y)`` intersection points to a dict with keys
/// ``"h"`` and ``"v"`` containing the edges that pass through that point.
#[pyfunction]
#[pyo3(name = "get_intersections_from_edges", signature = (h_edges, v_edges, tf_settings=None, **kwargs))]
fn py_get_intersections_from_edges(
    h_edges: Vec<Edge>,
    v_edges: Vec<Edge>,
    tf_settings: Option<TfSettings>,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyDict>> {
    let settings = if let Some(s) = tf_settings {
        Rc::new(s)
    } else {
        Rc::new(TfSettings::py_new(kwargs)?)
    };

    let intersections = TableFinder::new(settings).get_intersections_from_edges(h_edges, v_edges);

    Python::attach(|py| {
        let res = PyDict::new(py);
        for ((x, y), edge_map) in intersections {
            let h = edge_map
                .get(&Orientation::Horizontal)
                .cloned()
                .unwrap_or_default();
            let v = edge_map
                .get(&Orientation::Vertical)
                .cloned()
                .unwrap_or_default();
            let point_dict = PyDict::new(py);
            point_dict.set_item("h", h)?;
            point_dict.set_item("v", v)?;
            res.set_item((x.into_inner(), y.into_inner()), point_dict)?;
        }
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
/// Finds all table cell bounding boxes in a PDF page or from explicit edges.
///
/// # Arguments
///
/// * `page` - The PDF page to analyze. Can be None only if both
///   horizontal_strategy and vertical_strategy are set to "explicit".
/// * `clip` - Optional clip region (x1, y1, x2, y2). If provided, only edges
///   within this region are used for cell detection.
/// * `tf_settings` - Optional TableFinder settings object.
/// * `kwargs` - Optional keyword arguments for settings.
///
/// # Returns
///
/// A list of bounding boxes (x1, y1, x2, y2) for each detected cell.
///
/// # Raises
///
/// RuntimeError: If page is None and either strategy is not "explicit".
#[pyfunction]
#[pyo3(name="find_all_cells_bboxes", signature = (page=None, clip=None, tf_settings=None, **kwargs))]
fn py_find_all_cells_bboxes(
    page: Option<&Pyo3Page>,
    clip: Option<PyBbox>,
    tf_settings: Option<TfSettings>,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<Vec<PyBbox>> {
    let settings = if let Some(tf_settings) = tf_settings {
        Rc::new(tf_settings)
    } else {
        Rc::new(TfSettings::py_new(kwargs)?)
    };

    // Validate that page can only be None when both strategies are explicit
    if page.is_none()
        && (settings.horizontal_strategy != StrategyType::Explicit
            || settings.vertical_strategy != StrategyType::Explicit)
    {
        return Err(pyo3::exceptions::PyRuntimeError::new_err(
            "page can only be None when both horizontal_strategy and vertical_strategy are 'explicit'",
        ));
    }

    let page_ref = page.map(|p| &p.inner);
    let clip_bbox = clip.as_ref().map(py_bbox_to_rs_bbox);
    let cells = find_all_cells_bboxes(page_ref, settings.clone(), clip_bbox.as_ref());
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
/// * `need_strip` - Whether to strip leading/trailing whitespace from cell text (default: true).
/// * `kwargs` - Optional keyword arguments for settings.
///
/// # Returns
///
/// A list of Table objects constructed from the cells.
#[pyfunction]
#[pyo3(name = "find_tables_from_cells", signature = (cells, extract_text, page=None, tf_settings=None, **kwargs))]
fn py_find_tables_from_cells(
    cells: &Bound<'_, PyList>,
    extract_text: bool,
    page: Option<&Pyo3Page>,
    tf_settings: Option<TfSettings>,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<Vec<Table>> {
    let cells: Vec<BboxKey> = cells
        .iter()
        .map(|item| {
            let bbox: PyBbox = item.extract()?;
            Ok(py_bbox_to_rs_bbox(&bbox))
        })
        .collect::<PyResult<Vec<_>>>()?;
    let settings_value = match tf_settings {
        Some(s) => s,
        None => TfSettings::py_new(kwargs)?,
    };

    let page = match extract_text {
        true => match page {
            Some(page) => Some(&page.inner),
            None => {
                return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                    "page is required when extract_text is true",
                ));
            }
        },
        false => None,
    };

    let tables = find_tables_from_cells(&cells, extract_text, page, Some(&settings_value));
    Ok(tables)
}
/// Finds all tables in a PDF page.
///
/// # Arguments
///
/// * `page` - The PDF page to analyze. Can be None only if both strategies are explicit
///           and extract_text is false.
/// * `extract_text` - Whether to extract text content from table cells.
/// * `clip` - Optional clip region (x1, y1, x2, y2). If provided, only edges
///   within this region are used for table detection.
/// * `tf_settings` - Optional TableFinder settings object.
/// * `kwargs` - Optional keyword arguments for settings.
///
/// # Returns
///
/// A list of Table objects found in the page.
///
/// # Errors
///
/// Returns an error if:
/// - `page` is None and `extract_text` is true
/// - `page` is None and either strategy is not explicit
#[pyfunction]
#[pyo3(name = "find_tables", signature = (page=None, extract_text=true, clip=None, tf_settings=None, **kwargs))]
fn py_find_tables(
    page: Option<&Pyo3Page>,
    extract_text: bool,
    clip: Option<PyBbox>,
    tf_settings: Option<TfSettings>,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<Vec<Table>> {
    let settings = match tf_settings {
        Some(s) => Rc::new(s),
        None => Rc::new(TfSettings::py_new(kwargs)?),
    };

    // Validate: if extract_text is true, page must be provided
    if extract_text && page.is_none() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "page must be provided when extract_text is true",
        ));
    }

    // Validate: if page is None, both strategies must be explicit
    if page.is_none() {
        let h_strat = settings.horizontal_strategy;
        let v_strat = settings.vertical_strategy;
        if h_strat != StrategyType::Explicit || v_strat != StrategyType::Explicit {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "page can only be None when both horizontal_strategy and vertical_strategy are 'explicit'",
            ));
        }
    }

    let pdf_page = page.map(|p| &p.inner);
    let clip_bbox = clip.as_ref().map(py_bbox_to_rs_bbox);
    Ok(find_tables(
        pdf_page,
        settings,
        extract_text,
        clip_bbox.as_ref(),
    ))
}

/// Initializes the tablers Python module.
///
/// This function is called by Python when importing the module and registers
/// all classes and functions available to Python.
#[pymodule]
fn tablers(_py: Python<'_>, m: &Bound<PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<PdfiumRuntime>()?;
    m.add_class::<Pyo3Doc>()?;
    m.add_class::<Pyo3Page>()?;
    m.add_class::<PyPageIterator>()?;
    m.add_class::<Edge>()?;
    m.add_class::<TableCell>()?;
    m.add_class::<Table>()?;
    m.add_class::<PyCellGroup>()?;
    m.add_class::<PyTableCellValue>()?;
    m.add_class::<TfSettings>()?;
    m.add_class::<WordsExtractSettings>()?;
    m.add_function(pyo3::wrap_pyfunction!(py_find_all_cells_bboxes, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(py_find_tables_from_cells, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(py_find_tables, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(py_get_edges, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(py_get_intersections_from_edges, m)?)?;
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

    #[test]
    fn test_save_to_bytes_from_encrypted_pdf_matches_original() {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let pdfium = load_pdfium();
        let runtime = PdfiumRuntime::from_pdfium(pdfium);

        let pdf_path = format!(
            "{}/tests/data/test-encryption-pswd-qwerty.pdf",
            project_root
        );

        // Open the encrypted PDF with the correct password.
        let original = runtime
            .open_doc_from_path(&pdf_path, Some("qwerty"))
            .expect("Should open encrypted PDF with password");

        // Serialize via save_doc_without_security – the core logic behind Pyo3Doc::save_to_bytes.
        let stream_bytes = save_doc_without_security(pdfium, &original)
            .expect("Should serialize decrypted document to bytes without password");

        assert!(
            !stream_bytes.is_empty(),
            "Serialized stream should not be empty"
        );

        // The stream must be openable without any password.
        let from_stream = runtime
            .open_doc_from_bytes(&stream_bytes, None)
            .expect("Stream bytes should be openable without a password");

        // Page count must match.
        let original_page_count = original.pages().len();
        let stream_page_count = from_stream.pages().len();
        assert_eq!(
            original_page_count, stream_page_count,
            "Page count should match: original={original_page_count}, stream={stream_page_count}"
        );

        assert!(
            original_page_count > 0,
            "Document should have at least one page"
        );

        // Dimensions of every page must match.
        for idx in 0..original_page_count as usize {
            use pdfium_render::prelude::PdfPageIndex;
            let orig_page = original
                .pages()
                .get(idx as PdfPageIndex)
                .expect("Should get original page");
            let stream_page = from_stream
                .pages()
                .get(idx as PdfPageIndex)
                .expect("Should get stream page");

            let orig_w = orig_page.width().value;
            let orig_h = orig_page.height().value;
            let stream_w = stream_page.width().value;
            let stream_h = stream_page.height().value;

            assert_eq!(
                orig_w, stream_w,
                "Page {idx} width mismatch: original={orig_w}, stream={stream_w}"
            );
            assert_eq!(
                orig_h, stream_h,
                "Page {idx} height mismatch: original={orig_h}, stream={stream_h}"
            );
        }
    }

    #[test]
    fn test_pdfium_runtime_new_initializes_global() {
        let project_root = env!("CARGO_MANIFEST_DIR");

        #[cfg(target_os = "windows")]
        let pdfium_path = format!("{}/python/tablers/pdfium.dll", project_root);
        #[cfg(target_os = "macos")]
        let pdfium_path = format!("{}/python/tablers/libpdfium.dylib", project_root);
        #[cfg(target_os = "linux")]
        let pdfium_path = format!("{}/python/tablers/libpdfium.so.1", project_root);

        // Create a runtime using the public API
        let runtime = PdfiumRuntime::new(&pdfium_path);
        assert!(runtime.is_ok(), "Should successfully create PdfiumRuntime");

        // After first call, is_initialized should be true
        assert!(
            PdfiumRuntime::is_initialized(),
            "Should be initialized after new()"
        );
    }

    #[test]
    fn test_pdfium_runtime_get_returns_some_when_initialized() {
        let project_root = env!("CARGO_MANIFEST_DIR");

        #[cfg(target_os = "windows")]
        let pdfium_path = format!("{}/python/tablers/pdfium.dll", project_root);
        #[cfg(target_os = "macos")]
        let pdfium_path = format!("{}/python/tablers/libpdfium.dylib", project_root);
        #[cfg(target_os = "linux")]
        let pdfium_path = format!("{}/python/tablers/libpdfium.so.1", project_root);

        // Ensure initialized (may already be from another test)
        let _ = PdfiumRuntime::new(&pdfium_path);

        // get() should return Some
        let runtime = PdfiumRuntime::get();
        assert!(
            runtime.is_some(),
            "get() should return Some when initialized"
        );
    }

    #[test]
    fn test_pdfium_runtime_new_reuses_existing() {
        let project_root = env!("CARGO_MANIFEST_DIR");

        #[cfg(target_os = "windows")]
        let pdfium_path = format!("{}/python/tablers/pdfium.dll", project_root);
        #[cfg(target_os = "macos")]
        let pdfium_path = format!("{}/python/tablers/libpdfium.dylib", project_root);
        #[cfg(target_os = "linux")]
        let pdfium_path = format!("{}/python/tablers/libpdfium.so.1", project_root);

        // First call
        let runtime1 = PdfiumRuntime::new(&pdfium_path);
        assert!(runtime1.is_ok(), "First new() should succeed");

        // Second call with a different (non-existent) path should still succeed
        // because it reuses the existing instance
        let runtime2 = PdfiumRuntime::new("/nonexistent/path/to/pdfium.dll");
        assert!(
            runtime2.is_ok(),
            "Second new() should succeed by reusing existing instance"
        );
    }

    #[test]
    fn test_pdfium_runtime_can_open_document() {
        let project_root = env!("CARGO_MANIFEST_DIR");

        #[cfg(target_os = "windows")]
        let pdfium_path = format!("{}/python/tablers/pdfium.dll", project_root);
        #[cfg(target_os = "macos")]
        let pdfium_path = format!("{}/python/tablers/libpdfium.dylib", project_root);
        #[cfg(target_os = "linux")]
        let pdfium_path = format!("{}/python/tablers/libpdfium.so.1", project_root);

        let runtime = PdfiumRuntime::new(&pdfium_path).expect("Should create runtime");

        let pdf_path = format!("{}/tests/data/edge-test.pdf", project_root);
        let doc = runtime.open_doc_from_path(&pdf_path, None);

        assert!(doc.is_ok(), "Should open PDF document");
        let doc = doc.unwrap();
        assert!(doc.pages().len() > 0, "Document should have pages");
    }
}
