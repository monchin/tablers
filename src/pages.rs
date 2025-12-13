use crate::chars::Char;
use crate::edges::{Edge, EdgeType, make_edges};
use pdfium_render::prelude::PdfPage as PdfiumPage;
use std::cell::RefCell;
use std::collections::HashMap;

pub struct PdfPage {
    pub inner: PdfiumPage<'static>,
    pub edges: RefCell<Option<HashMap<EdgeType, Vec<Edge>>>>,
    pub chars: RefCell<Option<Vec<Char>>>,
    bottom_origin: bool,
}

impl PdfPage {
    pub fn new(inner: PdfiumPage<'static>, bottom_origin: bool) -> Self {
        Self {
            inner,
            edges: RefCell::new(None),
            chars: RefCell::new(None),
            bottom_origin,
        }
    }
    pub fn extract_edges(&self) {
        if self.edges.borrow().is_none() {
            let edges = make_edges(&self.inner, self.bottom_origin);
            self.edges.replace(Some(edges));
        }
    }

    pub fn clear(&self) {
        self.edges.replace(None);
        self.chars.replace(None);
    }

    pub fn width(&self) -> f32 {
        self.inner.width().value
    }

    pub fn height(&self) -> f32 {
        self.inner.height().value
    }
}
