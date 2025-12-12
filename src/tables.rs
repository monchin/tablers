use crate::edges::*;
use ordered_float::OrderedFloat;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StrategyType {
    Lines,
    LinesStrict,
    Text,
}

struct TfSettings {
    vertiacl_strategy: StrategyType,
    horizontal_strategy: StrategyType,
    snap_x_tolerance: OrderedFloat<f32>,
    snap_y_tolerance: OrderedFloat<f32>,
    join_x_tolerance: OrderedFloat<f32>,
    join_y_tolerance: OrderedFloat<f32>,
    edge_min_length: OrderedFloat<f32>,
    edge_min_length_prefilter: OrderedFloat<f32>,
    intersection_x_tolerance: OrderedFloat<f32>,
    intersection_y_tolerance: OrderedFloat<f32>,
}
fn filter_edges_by_min_len(edges: &mut Vec<Edge>, min_len: OrderedFloat<f32>) {
    edges.retain(|edge| match edge.edge_type {
        EdgeType::HorizontalLine => (edge.x2 - edge.x1) >= min_len,
        EdgeType::HorizontalRect => (edge.x2 - edge.x1) >= min_len,
        EdgeType::VerticalLine => (edge.y2 - edge.y1) >= min_len,
        EdgeType::VerticalRect => (edge.y2 - edge.y1) >= min_len,
    });
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
    fn get_edges(&self, page: &PdfPage) -> HashMap<Orientation, Vec<Edge>> {
        let settings = self.settings.as_ref();
        if (settings.vertiacl_strategy == StrategyType::Text)
            || (settings.horizontal_strategy == StrategyType::Text)
        {
            panic!("Text strategy not implemented")
        }

        let mut edges_all = make_edges(page, self.bottom_origin);
        let mut v_edges = match settings.vertiacl_strategy {
            StrategyType::LinesStrict => edges_all
                .remove(&EdgeType::VerticalLine)
                .unwrap_or_default(),
            StrategyType::Lines => [
                edges_all
                    .remove(&EdgeType::VerticalLine)
                    .unwrap_or_default(),
                edges_all
                    .remove(&EdgeType::VerticalRect)
                    .unwrap_or_default(),
            ]
            .concat(),
            _ => panic!("Text strategy not implemented"),
        };
        filter_edges_by_min_len(&mut v_edges, settings.edge_min_length_prefilter);
        let mut h_edges = match settings.horizontal_strategy {
            StrategyType::LinesStrict => edges_all
                .remove(&EdgeType::HorizontalLine)
                .unwrap_or_default(),
            StrategyType::Lines => [
                edges_all
                    .remove(&EdgeType::HorizontalLine)
                    .unwrap_or_default(),
                edges_all
                    .remove(&EdgeType::HorizontalRect)
                    .unwrap_or_default(),
            ]
            .concat(),
            _ => panic!("Text strategy not implemented"),
        };
        filter_edges_by_min_len(&mut h_edges, settings.edge_min_length_prefilter);

        let edges_prefiltered = HashMap::from([
            (Orientation::Vertical, v_edges),
            (Orientation::Horizontal, h_edges),
        ]);
        let mut edges_merged = merge_edges(
            edges_prefiltered,
            settings.snap_x_tolerance,
            settings.snap_y_tolerance,
            settings.join_x_tolerance,
            settings.join_y_tolerance,
        );
        if let Some(h_edges) = edges_merged.get_mut(&Orientation::Horizontal) {
            filter_edges_by_min_len(h_edges, settings.edge_min_length);
        }
        if let Some(v_edges) = edges_merged.get_mut(&Orientation::Vertical) {
            filter_edges_by_min_len(v_edges, settings.edge_min_length);
        }
        edges_merged
    }
}
