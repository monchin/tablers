use ordered_float::OrderedFloat;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::ops::BitOr;

static DEFAULT_SNAP_TOLERANCE: f32 = 3.0;
static DEFAULT_JOIN_TOLERANCE: f32 = 3.0;
static DEFAULT_INTERSECTION_TOLERANCE: f32 = 3.0;
static DEFAULT_MIN_WORDS_VERTICAL: usize = 3;
static DEFAULT_MIN_WORDS_HORIZONTAL: usize = 1;
static DEFAULT_X_TOLERANCE: f32 = 3.0;
static DEFAULT_Y_TOLERANCE: f32 = 3.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StrategyType {
    Lines = 1,
    LinesStrict = 2,
    Text = 4,
}

impl BitOr<u8> for StrategyType {
    type Output = u8;

    fn bitor(self, rhs: u8) -> Self::Output {
        (self as u8) | rhs
    }
}

impl BitOr<StrategyType> for StrategyType {
    type Output = u8;

    fn bitor(self, rhs: StrategyType) -> Self::Output {
        (self as u8) | (rhs as u8)
    }
}

#[derive(Debug, Clone)]
#[pyclass]
pub struct TfSettings {
    pub vertical_strategy: StrategyType,
    pub horizontal_strategy: StrategyType,
    pub snap_x_tolerance: OrderedFloat<f32>,
    pub snap_y_tolerance: OrderedFloat<f32>,
    pub join_x_tolerance: OrderedFloat<f32>,
    pub join_y_tolerance: OrderedFloat<f32>,
    pub edge_min_length: OrderedFloat<f32>,
    pub edge_min_length_prefilter: OrderedFloat<f32>,
    pub min_words_vertical: usize,
    pub min_words_horizontal: usize,
    pub intersection_x_tolerance: OrderedFloat<f32>,
    pub intersection_y_tolerance: OrderedFloat<f32>,
    pub text_settings: WordsExtractSettings,
}
impl Default for TfSettings {
    fn default() -> Self {
        TfSettings {
            vertical_strategy: StrategyType::Lines,
            horizontal_strategy: StrategyType::Lines,
            snap_x_tolerance: OrderedFloat::from(DEFAULT_SNAP_TOLERANCE),
            snap_y_tolerance: OrderedFloat::from(DEFAULT_SNAP_TOLERANCE),
            join_x_tolerance: OrderedFloat::from(DEFAULT_JOIN_TOLERANCE),
            join_y_tolerance: OrderedFloat::from(DEFAULT_JOIN_TOLERANCE),
            edge_min_length: OrderedFloat::from(3.0),
            edge_min_length_prefilter: OrderedFloat::from(1.0),
            min_words_vertical: DEFAULT_MIN_WORDS_VERTICAL,
            min_words_horizontal: DEFAULT_MIN_WORDS_HORIZONTAL,
            intersection_x_tolerance: OrderedFloat::from(DEFAULT_INTERSECTION_TOLERANCE),
            intersection_y_tolerance: OrderedFloat::from(DEFAULT_INTERSECTION_TOLERANCE),
            text_settings: WordsExtractSettings::default(),
        }
    }
}

#[pymethods]
impl TfSettings {
    #[new]
    #[pyo3(signature = (**kwargs))]
    pub fn py_new(kwargs: Option<&Bound<'_, PyDict>>) -> Self {
        let strategy_str_to_enum = |strategy_str: &str| -> StrategyType {
            match strategy_str {
                "lines" => StrategyType::Lines,
                "lines_strict" => StrategyType::LinesStrict,
                "text" => StrategyType::Text,
                _ => panic!("Invalid strategy: {}", strategy_str),
            }
        };
        let mut settings = TfSettings::default();

        if let Some(kwargs) = kwargs {
            for (key, value) in kwargs.iter() {
                let key = key.to_string();
                match key.as_str() {
                    "vertical_strategy" => {
                        settings.vertical_strategy = strategy_str_to_enum(value.extract().unwrap())
                    }
                    "horizontal_strategy" => {
                        settings.horizontal_strategy =
                            strategy_str_to_enum(value.extract().unwrap())
                    }
                    "snap_x_tolerance" => {
                        settings.snap_x_tolerance =
                            OrderedFloat::from(value.extract::<f32>().unwrap())
                    }
                    "snap_y_tolerance" => {
                        settings.snap_y_tolerance =
                            OrderedFloat::from(value.extract::<f32>().unwrap())
                    }
                    "join_x_tolerance" => {
                        settings.join_x_tolerance =
                            OrderedFloat::from(value.extract::<f32>().unwrap())
                    }
                    "join_y_tolerance" => {
                        settings.join_y_tolerance =
                            OrderedFloat::from(value.extract::<f32>().unwrap())
                    }
                    "edge_min_length" => {
                        settings.edge_min_length =
                            OrderedFloat::from(value.extract::<f32>().unwrap())
                    }
                    "edge_min_length_prefilter" => {
                        settings.edge_min_length_prefilter =
                            OrderedFloat::from(value.extract::<f32>().unwrap())
                    }
                    "min_words_vertical" => {
                        settings.min_words_vertical = value.extract::<usize>().unwrap()
                    }
                    "min_words_horizontal" => {
                        settings.min_words_horizontal = value.extract::<usize>().unwrap()
                    }
                    "intersection_x_tolerance" => {
                        settings.intersection_x_tolerance =
                            OrderedFloat::from(value.extract::<f32>().unwrap())
                    }
                    "intersection_y_tolerance" => {
                        settings.intersection_y_tolerance =
                            OrderedFloat::from(value.extract::<f32>().unwrap())
                    }
                    "text_x_tolerance" => {
                        settings.text_settings.x_tolerance =
                            OrderedFloat::from(value.extract::<f32>().unwrap())
                    }
                    "text_y_tolerance" => {
                        settings.text_settings.y_tolerance =
                            OrderedFloat::from(value.extract::<f32>().unwrap())
                    }
                    "text_keep_blank_chars" => {
                        settings.text_settings.keep_blank_chars = value.extract::<bool>().unwrap()
                    }
                    "text_use_text_flow" => {
                        settings.text_settings.use_text_flow = value.extract::<bool>().unwrap()
                    }
                    "text_read_in_clockwise" => {
                        settings.text_settings.text_read_in_clockwise = value.extract::<bool>().unwrap()
                    }
                    "text_split_at_punctuation" => {
                        let split_value: Option<&str> = value.extract().unwrap();
                        settings.text_settings.split_at_punctuation = match split_value {
                            Some("all") => Some(SplitPunctuation::All),
                            Some(custom) => Some(SplitPunctuation::Custom(custom.to_string())),
                            None => None,
                        };
                    }
                    "text_expand_ligatures" => {
                        settings.text_settings.expand_ligatures = value.extract::<bool>().unwrap()
                    }
                    _ => (), // Ignore unknown settings
                }
            }
        }
        settings
    }
}

#[derive(Debug, Clone)]
pub enum SplitPunctuation {
    All,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct WordsExtractSettings {
    pub x_tolerance: OrderedFloat<f32>,
    pub y_tolerance: OrderedFloat<f32>,
    pub keep_blank_chars: bool,
    pub use_text_flow: bool,
    pub text_read_in_clockwise: bool,
    pub split_at_punctuation: Option<SplitPunctuation>,
    pub expand_ligatures: bool,
}
impl Default for WordsExtractSettings {
    fn default() -> Self {
        WordsExtractSettings {
            x_tolerance: OrderedFloat::from(DEFAULT_X_TOLERANCE),
            y_tolerance: OrderedFloat::from(DEFAULT_Y_TOLERANCE),
            keep_blank_chars: false,
            use_text_flow: false,
            text_read_in_clockwise: true,
            split_at_punctuation: None,
            expand_ligatures: true,
        }
    }
}
