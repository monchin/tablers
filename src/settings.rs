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

// Helper methods for strategy conversion (not exposed to Python)
impl TfSettings {
    fn strategy_str_to_enum(strategy_str: &str) -> StrategyType {
        match strategy_str {
            "lines" => StrategyType::Lines,
            "lines_strict" => StrategyType::LinesStrict,
            "text" => StrategyType::Text,
            _ => panic!("Invalid strategy: {}", strategy_str),
        }
    }

    fn strategy_enum_to_str(strategy: StrategyType) -> &'static str {
        match strategy {
            StrategyType::Lines => "lines",
            StrategyType::LinesStrict => "lines_strict",
            StrategyType::Text => "text",
        }
    }
}

#[pymethods]
impl TfSettings {
    #[new]
    #[pyo3(signature = (**kwargs))]
    pub fn py_new(kwargs: Option<&Bound<'_, PyDict>>) -> Self {
        let mut settings = TfSettings::default();

        if let Some(kwargs) = kwargs {
            for (key, value) in kwargs.iter() {
                let key = key.to_string();
                match key.as_str() {
                    "vertical_strategy" => {
                        settings.vertical_strategy =
                            Self::strategy_str_to_enum(value.extract().unwrap())
                    }
                    "horizontal_strategy" => {
                        settings.horizontal_strategy =
                            Self::strategy_str_to_enum(value.extract().unwrap())
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
                        settings.text_settings.text_read_in_clockwise =
                            value.extract::<bool>().unwrap()
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

    // Getters
    #[getter]
    fn vertical_strategy(&self) -> &'static str {
        Self::strategy_enum_to_str(self.vertical_strategy)
    }

    #[getter]
    fn horizontal_strategy(&self) -> &'static str {
        Self::strategy_enum_to_str(self.horizontal_strategy)
    }

    #[getter]
    fn snap_x_tolerance(&self) -> f32 {
        self.snap_x_tolerance.into_inner()
    }

    #[getter]
    fn snap_y_tolerance(&self) -> f32 {
        self.snap_y_tolerance.into_inner()
    }

    #[getter]
    fn join_x_tolerance(&self) -> f32 {
        self.join_x_tolerance.into_inner()
    }

    #[getter]
    fn join_y_tolerance(&self) -> f32 {
        self.join_y_tolerance.into_inner()
    }

    #[getter]
    fn edge_min_length(&self) -> f32 {
        self.edge_min_length.into_inner()
    }

    #[getter]
    fn edge_min_length_prefilter(&self) -> f32 {
        self.edge_min_length_prefilter.into_inner()
    }

    #[getter]
    fn min_words_vertical(&self) -> usize {
        self.min_words_vertical
    }

    #[getter]
    fn min_words_horizontal(&self) -> usize {
        self.min_words_horizontal
    }

    #[getter]
    fn intersection_x_tolerance(&self) -> f32 {
        self.intersection_x_tolerance.into_inner()
    }

    #[getter]
    fn intersection_y_tolerance(&self) -> f32 {
        self.intersection_y_tolerance.into_inner()
    }

    #[getter]
    fn text_settings(&self) -> WordsExtractSettings {
        self.text_settings.clone()
    }

    #[getter]
    fn text_x_tolerance(&self) -> f32 {
        self.text_settings.x_tolerance.into_inner()
    }

    #[getter]
    fn text_y_tolerance(&self) -> f32 {
        self.text_settings.y_tolerance.into_inner()
    }

    #[getter]
    fn text_keep_blank_chars(&self) -> bool {
        self.text_settings.keep_blank_chars
    }

    #[getter]
    fn text_use_text_flow(&self) -> bool {
        self.text_settings.use_text_flow
    }

    #[getter]
    fn text_read_in_clockwise(&self) -> bool {
        self.text_settings.text_read_in_clockwise
    }

    #[getter]
    fn text_split_at_punctuation(&self) -> Option<String> {
        match &self.text_settings.split_at_punctuation {
            Some(SplitPunctuation::All) => Some("all".to_string()),
            Some(SplitPunctuation::Custom(s)) => Some(s.clone()),
            None => None,
        }
    }

    #[getter]
    fn text_expand_ligatures(&self) -> bool {
        self.text_settings.expand_ligatures
    }

    // Setters
    #[setter]
    fn set_vertical_strategy(&mut self, value: &str) {
        self.vertical_strategy = Self::strategy_str_to_enum(value);
    }

    #[setter]
    fn set_horizontal_strategy(&mut self, value: &str) {
        self.horizontal_strategy = Self::strategy_str_to_enum(value);
    }

    #[setter]
    fn set_snap_x_tolerance(&mut self, value: f32) {
        self.snap_x_tolerance = OrderedFloat::from(value);
    }

    #[setter]
    fn set_snap_y_tolerance(&mut self, value: f32) {
        self.snap_y_tolerance = OrderedFloat::from(value);
    }

    #[setter]
    fn set_join_x_tolerance(&mut self, value: f32) {
        self.join_x_tolerance = OrderedFloat::from(value);
    }

    #[setter]
    fn set_join_y_tolerance(&mut self, value: f32) {
        self.join_y_tolerance = OrderedFloat::from(value);
    }

    #[setter]
    fn set_edge_min_length(&mut self, value: f32) {
        self.edge_min_length = OrderedFloat::from(value);
    }

    #[setter]
    fn set_edge_min_length_prefilter(&mut self, value: f32) {
        self.edge_min_length_prefilter = OrderedFloat::from(value);
    }

    #[setter]
    fn set_min_words_vertical(&mut self, value: usize) {
        self.min_words_vertical = value;
    }

    #[setter]
    fn set_min_words_horizontal(&mut self, value: usize) {
        self.min_words_horizontal = value;
    }

    #[setter]
    fn set_intersection_x_tolerance(&mut self, value: f32) {
        self.intersection_x_tolerance = OrderedFloat::from(value);
    }

    #[setter]
    fn set_intersection_y_tolerance(&mut self, value: f32) {
        self.intersection_y_tolerance = OrderedFloat::from(value);
    }

    #[setter]
    fn set_text_settings(&mut self, value: WordsExtractSettings) {
        self.text_settings = value;
    }

    #[setter]
    fn set_text_x_tolerance(&mut self, value: f32) {
        self.text_settings.x_tolerance = OrderedFloat::from(value);
    }

    #[setter]
    fn set_text_y_tolerance(&mut self, value: f32) {
        self.text_settings.y_tolerance = OrderedFloat::from(value);
    }

    #[setter]
    fn set_text_keep_blank_chars(&mut self, value: bool) {
        self.text_settings.keep_blank_chars = value;
    }

    #[setter]
    fn set_text_use_text_flow(&mut self, value: bool) {
        self.text_settings.use_text_flow = value;
    }

    #[setter]
    fn set_text_read_in_clockwise(&mut self, value: bool) {
        self.text_settings.text_read_in_clockwise = value;
    }

    #[setter]
    fn set_text_split_at_punctuation(&mut self, value: Option<&str>) {
        self.text_settings.split_at_punctuation = match value {
            Some("all") => Some(SplitPunctuation::All),
            Some(custom) => Some(SplitPunctuation::Custom(custom.to_string())),
            None => None,
        };
    }

    #[setter]
    fn set_text_expand_ligatures(&mut self, value: bool) {
        self.text_settings.expand_ligatures = value;
    }

    // Dataclass-like methods
    fn __repr__(&self) -> String {
        format!(
            "TfSettings(vertical_strategy='{}', horizontal_strategy='{}', \
             snap_x_tolerance={}, snap_y_tolerance={}, \
             join_x_tolerance={}, join_y_tolerance={}, \
             edge_min_length={}, edge_min_length_prefilter={}, \
             min_words_vertical={}, min_words_horizontal={}, \
             intersection_x_tolerance={}, intersection_y_tolerance={}, \
             text_x_tolerance={}, text_y_tolerance={}, \
             text_keep_blank_chars={}, text_use_text_flow={}, \
             text_read_in_clockwise={}, text_split_at_punctuation={:?}, \
             text_expand_ligatures={})",
            Self::strategy_enum_to_str(self.vertical_strategy),
            Self::strategy_enum_to_str(self.horizontal_strategy),
            self.snap_x_tolerance,
            self.snap_y_tolerance,
            self.join_x_tolerance,
            self.join_y_tolerance,
            self.edge_min_length,
            self.edge_min_length_prefilter,
            self.min_words_vertical,
            self.min_words_horizontal,
            self.intersection_x_tolerance,
            self.intersection_y_tolerance,
            self.text_settings.x_tolerance,
            self.text_settings.y_tolerance,
            self.text_settings.keep_blank_chars,
            self.text_settings.use_text_flow,
            self.text_settings.text_read_in_clockwise,
            self.text_split_at_punctuation(),
            self.text_settings.expand_ligatures,
        )
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        if let Ok(other) = other.extract::<TfSettings>() {
            self.vertical_strategy == other.vertical_strategy
                && self.horizontal_strategy == other.horizontal_strategy
                && self.snap_x_tolerance == other.snap_x_tolerance
                && self.snap_y_tolerance == other.snap_y_tolerance
                && self.join_x_tolerance == other.join_x_tolerance
                && self.join_y_tolerance == other.join_y_tolerance
                && self.edge_min_length == other.edge_min_length
                && self.edge_min_length_prefilter == other.edge_min_length_prefilter
                && self.min_words_vertical == other.min_words_vertical
                && self.min_words_horizontal == other.min_words_horizontal
                && self.intersection_x_tolerance == other.intersection_x_tolerance
                && self.intersection_y_tolerance == other.intersection_y_tolerance
                && self.text_settings.x_tolerance == other.text_settings.x_tolerance
                && self.text_settings.y_tolerance == other.text_settings.y_tolerance
                && self.text_settings.keep_blank_chars == other.text_settings.keep_blank_chars
                && self.text_settings.use_text_flow == other.text_settings.use_text_flow
                && self.text_settings.text_read_in_clockwise
                    == other.text_settings.text_read_in_clockwise
                && self.text_settings.expand_ligatures == other.text_settings.expand_ligatures
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub enum SplitPunctuation {
    All,
    Custom(String),
}

#[derive(Debug, Clone)]
#[pyclass]
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

// Helper methods for WordsExtractSettings (not exposed to Python)
impl WordsExtractSettings {
    fn split_punctuation_to_str(&self) -> Option<String> {
        match &self.split_at_punctuation {
            Some(SplitPunctuation::All) => Some("all".to_string()),
            Some(SplitPunctuation::Custom(s)) => Some(s.clone()),
            None => None,
        }
    }

    fn str_to_split_punctuation(value: Option<&str>) -> Option<SplitPunctuation> {
        match value {
            Some("all") => Some(SplitPunctuation::All),
            Some(custom) => Some(SplitPunctuation::Custom(custom.to_string())),
            None => None,
        }
    }
}

#[pymethods]
impl WordsExtractSettings {
    #[new]
    #[pyo3(signature = (**kwargs))]
    pub fn py_new(kwargs: Option<&Bound<'_, PyDict>>) -> Self {
        let mut settings = WordsExtractSettings::default();

        if let Some(kwargs) = kwargs {
            for (key, value) in kwargs.iter() {
                let key = key.to_string();
                match key.as_str() {
                    "x_tolerance" => {
                        settings.x_tolerance = OrderedFloat::from(value.extract::<f32>().unwrap())
                    }
                    "y_tolerance" => {
                        settings.y_tolerance = OrderedFloat::from(value.extract::<f32>().unwrap())
                    }
                    "keep_blank_chars" => {
                        settings.keep_blank_chars = value.extract::<bool>().unwrap()
                    }
                    "use_text_flow" => settings.use_text_flow = value.extract::<bool>().unwrap(),
                    "text_read_in_clockwise" => {
                        settings.text_read_in_clockwise = value.extract::<bool>().unwrap()
                    }
                    "split_at_punctuation" => {
                        let split_value: Option<&str> = value.extract().unwrap();
                        settings.split_at_punctuation = Self::str_to_split_punctuation(split_value);
                    }
                    "expand_ligatures" => {
                        settings.expand_ligatures = value.extract::<bool>().unwrap()
                    }
                    _ => (), // Ignore unknown settings
                }
            }
        }
        settings
    }

    // Getters
    #[getter]
    fn x_tolerance(&self) -> f32 {
        self.x_tolerance.into_inner()
    }

    #[getter]
    fn y_tolerance(&self) -> f32 {
        self.y_tolerance.into_inner()
    }

    #[getter]
    fn keep_blank_chars(&self) -> bool {
        self.keep_blank_chars
    }

    #[getter]
    fn use_text_flow(&self) -> bool {
        self.use_text_flow
    }

    #[getter]
    fn text_read_in_clockwise(&self) -> bool {
        self.text_read_in_clockwise
    }

    #[getter]
    fn split_at_punctuation(&self) -> Option<String> {
        self.split_punctuation_to_str()
    }

    #[getter]
    fn expand_ligatures(&self) -> bool {
        self.expand_ligatures
    }

    // Setters
    #[setter]
    fn set_x_tolerance(&mut self, value: f32) {
        self.x_tolerance = OrderedFloat::from(value);
    }

    #[setter]
    fn set_y_tolerance(&mut self, value: f32) {
        self.y_tolerance = OrderedFloat::from(value);
    }

    #[setter]
    fn set_keep_blank_chars(&mut self, value: bool) {
        self.keep_blank_chars = value;
    }

    #[setter]
    fn set_use_text_flow(&mut self, value: bool) {
        self.use_text_flow = value;
    }

    #[setter]
    fn set_text_read_in_clockwise(&mut self, value: bool) {
        self.text_read_in_clockwise = value;
    }

    #[setter]
    fn set_split_at_punctuation(&mut self, value: Option<&str>) {
        self.split_at_punctuation = Self::str_to_split_punctuation(value);
    }

    #[setter]
    fn set_expand_ligatures(&mut self, value: bool) {
        self.expand_ligatures = value;
    }

    // Dataclass-like methods
    fn __repr__(&self) -> String {
        format!(
            "WordsExtractSettings(x_tolerance={}, y_tolerance={}, \
             keep_blank_chars={}, use_text_flow={}, \
             text_read_in_clockwise={}, split_at_punctuation={:?}, \
             expand_ligatures={})",
            self.x_tolerance,
            self.y_tolerance,
            self.keep_blank_chars,
            self.use_text_flow,
            self.text_read_in_clockwise,
            self.split_punctuation_to_str(),
            self.expand_ligatures,
        )
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        if let Ok(other) = other.extract::<WordsExtractSettings>() {
            self.x_tolerance == other.x_tolerance
                && self.y_tolerance == other.y_tolerance
                && self.keep_blank_chars == other.keep_blank_chars
                && self.use_text_flow == other.use_text_flow
                && self.text_read_in_clockwise == other.text_read_in_clockwise
                && self.expand_ligatures == other.expand_ligatures
        } else {
            false
        }
    }
}
