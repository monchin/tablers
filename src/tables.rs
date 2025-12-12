use crate::edges::*;
use pdfium_render::prelude::*;
use std::collections::HashMap;
use std::rc::Rc;

// use crate::edges::*;
struct Cell {
    text: String,
    bbox: PdfRect,
}
struct Table {
    cells: Vec<Cell>,
    bbox: PdfRect,
    page_index: usize, 
}

enum StrategyType {
    Lines,
    LinesStrict,
    Text,
}
struct TfSettings {
    vertiacl_strategy: StrategyType,
    horizontal_strategy: StrategyType,
}

struct TableFinder {
    bottom_origin: bool,
    settings: Rc<TfSettings>,
}

impl TableFinder {
    fn new(settings: Rc<TfSettings>, bottom_origin: bool) -> Self {
        TableFinder {
            bottom_origin: bottom_origin,
            settings: settings.clone(),
        }
    }
//     fn get_edges(&self, page: &PdfPage) -> HashMap<EdgeType, Vec<Edge>> {
//         // make_edges(page, self.bottom_origin);
//     }
}
