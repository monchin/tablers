use ordered_float::OrderedFloat;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::fmt;
use std::ops::{BitAnd, Deref};
use thiserror::Error;

/// Error type for invalid non-negative float values.
#[derive(Debug, Clone, Error)]
#[error("{field_name} must be non-negative, got {value}")]
pub struct NegativeValueError {
    pub field_name: String,
    pub value: f32,
}

impl NegativeValueError {
    pub fn new(field_name: impl Into<String>, value: f32) -> Self {
        Self {
            field_name: field_name.into(),
            value,
        }
    }
}

impl From<NegativeValueError> for PyErr {
    fn from(err: NegativeValueError) -> PyErr {
        PyValueError::new_err(err.to_string())
    }
}

/// A non-negative floating point number wrapper.
///
/// This type ensures that the wrapped value is always >= 0.0.
/// It wraps `OrderedFloat<f32>` to maintain ordering capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonNegativeF32(OrderedFloat<f32>);

impl NonNegativeF32 {
    /// Creates a new NonNegativeF32 from a f32 value.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to wrap.
    /// * `field_name` - The name of the field (for error messages).
    ///
    /// # Returns
    ///
    /// Ok(NonNegativeF32) if value >= 0, Err(NegativeValueError) otherwise.
    pub fn new(value: f32, field_name: impl Into<String>) -> Result<Self, NegativeValueError> {
        if value < 0.0 {
            Err(NegativeValueError::new(field_name, value))
        } else {
            Ok(Self(OrderedFloat::from(value)))
        }
    }

    /// Creates a new NonNegativeF32 without validation.
    ///
    /// # Safety
    ///
    /// The caller must ensure that value >= 0.0.
    #[inline]
    pub const fn new_unchecked(value: f32) -> Self {
        Self(OrderedFloat(value))
    }

    /// Returns the inner f32 value.
    #[inline]
    pub fn into_inner(self) -> f32 {
        self.0.into_inner()
    }

    /// Returns the inner OrderedFloat value.
    #[inline]
    pub fn as_ordered_float(self) -> OrderedFloat<f32> {
        self.0
    }
}

impl fmt::Display for NonNegativeF32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<NonNegativeF32> for f32 {
    fn from(val: NonNegativeF32) -> Self {
        val.0.into_inner()
    }
}

impl From<NonNegativeF32> for OrderedFloat<f32> {
    fn from(val: NonNegativeF32) -> Self {
        val.0
    }
}

impl Deref for NonNegativeF32 {
    type Target = OrderedFloat<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Default tolerance for snapping nearby edges together.
static DEFAULT_SNAP_TOLERANCE: f32 = 3.0;
/// Default tolerance for joining overlapping edges.
static DEFAULT_JOIN_TOLERANCE: f32 = 3.0;
/// Default tolerance for detecting edge intersections.
static DEFAULT_INTERSECTION_TOLERANCE: f32 = 3.0;
/// Default minimum words required for vertical text-based edge detection.
static DEFAULT_MIN_WORDS_VERTICAL: usize = 3;
/// Default minimum words required for horizontal text-based edge detection.
static DEFAULT_MIN_WORDS_HORIZONTAL: usize = 1;
/// Default x-tolerance for word extraction.
static DEFAULT_X_TOLERANCE: f32 = 3.0;
/// Default y-tolerance for word extraction.
static DEFAULT_Y_TOLERANCE: f32 = 3.0;

/// Strategy types for edge detection in table finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StrategyType {
    /// Use visible lines and rectangle borders.
    Lines = 1,
    /// Use only explicit line objects (stricter than Lines).
    LinesStrict = 2,
    /// Infer edges from text alignment.
    Text = 4,
}

impl BitAnd<u8> for StrategyType {
    type Output = u8;

    fn bitand(self, rhs: u8) -> Self::Output {
        (self as u8) & rhs
    }
}

impl BitAnd<StrategyType> for StrategyType {
    type Output = u8;

    fn bitand(self, rhs: StrategyType) -> Self::Output {
        (self as u8) & (rhs as u8)
    }
}

/// Settings for table finding operations.
///
/// Controls how edges are detected, snapped, joined, and how intersections
/// are identified when finding tables in a PDF page.
#[derive(Debug, Clone)]
#[pyclass]
pub struct TfSettings {
    /// Strategy for detecting vertical edges.
    pub vertical_strategy: StrategyType,
    /// Strategy for detecting horizontal edges.
    pub horizontal_strategy: StrategyType,
    /// Tolerance for snapping vertical edges together.
    pub snap_x_tolerance: NonNegativeF32,
    /// Tolerance for snapping horizontal edges together.
    pub snap_y_tolerance: NonNegativeF32,
    /// Tolerance for joining horizontal edges.
    pub join_x_tolerance: NonNegativeF32,
    /// Tolerance for joining vertical edges.
    pub join_y_tolerance: NonNegativeF32,
    /// Minimum length for edges to be included.
    pub edge_min_length: NonNegativeF32,
    /// Minimum length for edges before merging.
    pub edge_min_length_prefilter: NonNegativeF32,
    /// Minimum words for vertical text-based edge detection.
    pub min_words_vertical: usize,
    /// Minimum words for horizontal text-based edge detection.
    pub min_words_horizontal: usize,
    /// X-tolerance for detecting edge intersections.
    pub intersection_x_tolerance: NonNegativeF32,
    /// Y-tolerance for detecting edge intersections.
    pub intersection_y_tolerance: NonNegativeF32,
    /// Settings for text/word extraction.
    pub text_settings: WordsExtractSettings,
}
impl Default for TfSettings {
    /// Creates a TfSettings instance with default values.
    fn default() -> Self {
        TfSettings {
            vertical_strategy: StrategyType::LinesStrict, // LinesStrict is more intuitive for default behavior
            horizontal_strategy: StrategyType::LinesStrict,
            snap_x_tolerance: NonNegativeF32::new_unchecked(DEFAULT_SNAP_TOLERANCE),
            snap_y_tolerance: NonNegativeF32::new_unchecked(DEFAULT_SNAP_TOLERANCE),
            join_x_tolerance: NonNegativeF32::new_unchecked(DEFAULT_JOIN_TOLERANCE),
            join_y_tolerance: NonNegativeF32::new_unchecked(DEFAULT_JOIN_TOLERANCE),
            edge_min_length: NonNegativeF32::new_unchecked(3.0),
            edge_min_length_prefilter: NonNegativeF32::new_unchecked(1.0),
            min_words_vertical: DEFAULT_MIN_WORDS_VERTICAL,
            min_words_horizontal: DEFAULT_MIN_WORDS_HORIZONTAL,
            intersection_x_tolerance: NonNegativeF32::new_unchecked(DEFAULT_INTERSECTION_TOLERANCE),
            intersection_y_tolerance: NonNegativeF32::new_unchecked(DEFAULT_INTERSECTION_TOLERANCE),
            text_settings: WordsExtractSettings::default(),
        }
    }
}

/// Helper methods for strategy conversion (not exposed to Python).
impl TfSettings {
    /// Converts a strategy string to its enum representation.
    ///
    /// # Arguments
    ///
    /// * `strategy_str` - The strategy name ("lines", "lines_strict", or "text").
    ///
    /// # Returns
    ///
    /// The corresponding StrategyType enum value.
    ///
    /// # Panics
    ///
    /// Panics if an invalid strategy string is provided.
    fn strategy_str_to_enum(strategy_str: &str) -> StrategyType {
        match strategy_str {
            "lines" => StrategyType::Lines,
            "lines_strict" => StrategyType::LinesStrict,
            "text" => StrategyType::Text,
            _ => panic!("Invalid strategy: {}", strategy_str),
        }
    }

    /// Converts a StrategyType enum to its string representation.
    ///
    /// # Arguments
    ///
    /// * `strategy` - The strategy enum value.
    ///
    /// # Returns
    ///
    /// The string name of the strategy.
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
    /// Creates a new TfSettings instance from keyword arguments.
    ///
    /// # Arguments
    ///
    /// * `kwargs` - Optional dictionary of settings to override defaults.
    ///
    /// # Returns
    ///
    /// A new TfSettings instance.
    ///
    /// # Errors
    ///
    /// Returns PyValueError if any numeric value is negative.
    #[new]
    #[pyo3(signature = (**kwargs))]
    pub fn py_new(kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
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
                        let v = value.extract::<f32>().unwrap();
                        settings.snap_x_tolerance = NonNegativeF32::new(v, "snap_x_tolerance")?;
                    }
                    "snap_y_tolerance" => {
                        let v = value.extract::<f32>().unwrap();
                        settings.snap_y_tolerance = NonNegativeF32::new(v, "snap_y_tolerance")?;
                    }
                    "join_x_tolerance" => {
                        let v = value.extract::<f32>().unwrap();
                        settings.join_x_tolerance = NonNegativeF32::new(v, "join_x_tolerance")?;
                    }
                    "join_y_tolerance" => {
                        let v = value.extract::<f32>().unwrap();
                        settings.join_y_tolerance = NonNegativeF32::new(v, "join_y_tolerance")?;
                    }
                    "edge_min_length" => {
                        let v = value.extract::<f32>().unwrap();
                        settings.edge_min_length = NonNegativeF32::new(v, "edge_min_length")?;
                    }
                    "edge_min_length_prefilter" => {
                        let v = value.extract::<f32>().unwrap();
                        settings.edge_min_length_prefilter =
                            NonNegativeF32::new(v, "edge_min_length_prefilter")?;
                    }
                    "min_words_vertical" => {
                        settings.min_words_vertical = value.extract::<usize>().unwrap()
                    }
                    "min_words_horizontal" => {
                        settings.min_words_horizontal = value.extract::<usize>().unwrap()
                    }
                    "intersection_x_tolerance" => {
                        let v = value.extract::<f32>().unwrap();
                        settings.intersection_x_tolerance =
                            NonNegativeF32::new(v, "intersection_x_tolerance")?;
                    }
                    "intersection_y_tolerance" => {
                        let v = value.extract::<f32>().unwrap();
                        settings.intersection_y_tolerance =
                            NonNegativeF32::new(v, "intersection_y_tolerance")?;
                    }
                    "text_x_tolerance" => {
                        let v = value.extract::<f32>().unwrap();
                        settings.text_settings.x_tolerance =
                            NonNegativeF32::new(v, "text_x_tolerance")?;
                    }
                    "text_y_tolerance" => {
                        let v = value.extract::<f32>().unwrap();
                        settings.text_settings.y_tolerance =
                            NonNegativeF32::new(v, "text_y_tolerance")?;
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
        Ok(settings)
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
    fn set_snap_x_tolerance(&mut self, value: f32) -> PyResult<()> {
        self.snap_x_tolerance = NonNegativeF32::new(value, "snap_x_tolerance")?;
        Ok(())
    }

    #[setter]
    fn set_snap_y_tolerance(&mut self, value: f32) -> PyResult<()> {
        self.snap_y_tolerance = NonNegativeF32::new(value, "snap_y_tolerance")?;
        Ok(())
    }

    #[setter]
    fn set_join_x_tolerance(&mut self, value: f32) -> PyResult<()> {
        self.join_x_tolerance = NonNegativeF32::new(value, "join_x_tolerance")?;
        Ok(())
    }

    #[setter]
    fn set_join_y_tolerance(&mut self, value: f32) -> PyResult<()> {
        self.join_y_tolerance = NonNegativeF32::new(value, "join_y_tolerance")?;
        Ok(())
    }

    #[setter]
    fn set_edge_min_length(&mut self, value: f32) -> PyResult<()> {
        self.edge_min_length = NonNegativeF32::new(value, "edge_min_length")?;
        Ok(())
    }

    #[setter]
    fn set_edge_min_length_prefilter(&mut self, value: f32) -> PyResult<()> {
        self.edge_min_length_prefilter = NonNegativeF32::new(value, "edge_min_length_prefilter")?;
        Ok(())
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
    fn set_intersection_x_tolerance(&mut self, value: f32) -> PyResult<()> {
        self.intersection_x_tolerance = NonNegativeF32::new(value, "intersection_x_tolerance")?;
        Ok(())
    }

    #[setter]
    fn set_intersection_y_tolerance(&mut self, value: f32) -> PyResult<()> {
        self.intersection_y_tolerance = NonNegativeF32::new(value, "intersection_y_tolerance")?;
        Ok(())
    }

    #[setter]
    fn set_text_settings(&mut self, value: WordsExtractSettings) {
        self.text_settings = value;
    }

    #[setter]
    fn set_text_x_tolerance(&mut self, value: f32) -> PyResult<()> {
        self.text_settings.x_tolerance = NonNegativeF32::new(value, "text_x_tolerance")?;
        Ok(())
    }

    #[setter]
    fn set_text_y_tolerance(&mut self, value: f32) -> PyResult<()> {
        self.text_settings.y_tolerance = NonNegativeF32::new(value, "text_y_tolerance")?;
        Ok(())
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

/// Specifies how to split words at punctuation characters.
#[derive(Debug, Clone)]
pub enum SplitPunctuation {
    /// Split at all standard punctuation characters.
    All,
    /// Split at a custom set of characters.
    Custom(String),
}

/// Settings for word extraction from PDF text.
///
/// Controls how characters are grouped into words, including
/// tolerance values and text direction handling.
#[derive(Debug, Clone)]
#[pyclass]
pub struct WordsExtractSettings {
    /// X-axis tolerance for grouping characters into words.
    pub x_tolerance: NonNegativeF32,
    /// Y-axis tolerance for grouping characters into lines.
    pub y_tolerance: NonNegativeF32,
    /// Whether to preserve blank/whitespace characters.
    pub keep_blank_chars: bool,
    /// Whether to use the PDF's text flow order.
    pub use_text_flow: bool,
    /// Whether text reads in clockwise direction.
    pub text_read_in_clockwise: bool,
    /// Optional punctuation splitting configuration.
    pub split_at_punctuation: Option<SplitPunctuation>,
    /// Whether to expand ligatures into individual characters.
    pub expand_ligatures: bool,
}

impl Default for WordsExtractSettings {
    /// Creates a WordsExtractSettings instance with default values.
    fn default() -> Self {
        WordsExtractSettings {
            x_tolerance: NonNegativeF32::new_unchecked(DEFAULT_X_TOLERANCE),
            y_tolerance: NonNegativeF32::new_unchecked(DEFAULT_Y_TOLERANCE),
            keep_blank_chars: false,
            use_text_flow: false,
            text_read_in_clockwise: true,
            split_at_punctuation: None,
            expand_ligatures: true,
        }
    }
}

/// Helper methods for WordsExtractSettings (not exposed to Python).
impl WordsExtractSettings {
    /// Converts the split_at_punctuation setting to a string.
    fn split_punctuation_to_str(&self) -> Option<String> {
        match &self.split_at_punctuation {
            Some(SplitPunctuation::All) => Some("all".to_string()),
            Some(SplitPunctuation::Custom(s)) => Some(s.clone()),
            None => None,
        }
    }

    /// Converts a string to SplitPunctuation setting.
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
    /// Creates a new WordsExtractSettings instance from keyword arguments.
    ///
    /// # Arguments
    ///
    /// * `kwargs` - Optional dictionary of settings to override defaults.
    ///
    /// # Returns
    ///
    /// A new WordsExtractSettings instance.
    ///
    /// # Errors
    ///
    /// Returns PyValueError if any numeric value is negative.
    #[new]
    #[pyo3(signature = (**kwargs))]
    pub fn py_new(kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        let mut settings = WordsExtractSettings::default();

        if let Some(kwargs) = kwargs {
            for (key, value) in kwargs.iter() {
                let key = key.to_string();
                match key.as_str() {
                    "x_tolerance" => {
                        let v = value.extract::<f32>().unwrap();
                        settings.x_tolerance = NonNegativeF32::new(v, "x_tolerance")?;
                    }
                    "y_tolerance" => {
                        let v = value.extract::<f32>().unwrap();
                        settings.y_tolerance = NonNegativeF32::new(v, "y_tolerance")?;
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
        Ok(settings)
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
    fn set_x_tolerance(&mut self, value: f32) -> PyResult<()> {
        self.x_tolerance = NonNegativeF32::new(value, "x_tolerance")?;
        Ok(())
    }

    #[setter]
    fn set_y_tolerance(&mut self, value: f32) -> PyResult<()> {
        self.y_tolerance = NonNegativeF32::new(value, "y_tolerance")?;
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    // TfSettings tests
    #[test]
    fn test_tf_settings_default() {
        let settings = TfSettings::default();
        assert_eq!(settings.vertical_strategy, StrategyType::LinesStrict);
        assert_eq!(settings.horizontal_strategy, StrategyType::LinesStrict);
        assert_eq!(settings.snap_x_tolerance.into_inner(), 3.0);
        assert_eq!(settings.snap_y_tolerance.into_inner(), 3.0);
        assert_eq!(settings.join_x_tolerance.into_inner(), 3.0);
        assert_eq!(settings.join_y_tolerance.into_inner(), 3.0);
        assert_eq!(settings.edge_min_length.into_inner(), 3.0);
        assert_eq!(settings.edge_min_length_prefilter.into_inner(), 1.0);
        assert_eq!(settings.min_words_vertical, 3);
        assert_eq!(settings.min_words_horizontal, 1);
        assert_eq!(settings.intersection_x_tolerance.into_inner(), 3.0);
        assert_eq!(settings.intersection_y_tolerance.into_inner(), 3.0);
    }

    #[test]
    fn test_non_negative_f32_valid() {
        let val = NonNegativeF32::new(3.0, "test").unwrap();
        assert_eq!(val.into_inner(), 3.0);

        let zero = NonNegativeF32::new(0.0, "test").unwrap();
        assert_eq!(zero.into_inner(), 0.0);
    }

    #[test]
    fn test_non_negative_f32_invalid() {
        let result = NonNegativeF32::new(-1.0, "test_field");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.field_name, "test_field");
        assert_eq!(err.value, -1.0);
    }

    #[test]
    fn test_strategy_str_to_enum() {
        assert_eq!(
            TfSettings::strategy_str_to_enum("lines"),
            StrategyType::Lines
        );
        assert_eq!(
            TfSettings::strategy_str_to_enum("lines_strict"),
            StrategyType::LinesStrict
        );
        assert_eq!(TfSettings::strategy_str_to_enum("text"), StrategyType::Text);
    }

    #[test]
    #[should_panic(expected = "Invalid strategy")]
    fn test_strategy_str_to_enum_invalid() {
        TfSettings::strategy_str_to_enum("invalid");
    }

    #[test]
    fn test_strategy_enum_to_str() {
        assert_eq!(
            TfSettings::strategy_enum_to_str(StrategyType::Lines),
            "lines"
        );
        assert_eq!(
            TfSettings::strategy_enum_to_str(StrategyType::LinesStrict),
            "lines_strict"
        );
        assert_eq!(TfSettings::strategy_enum_to_str(StrategyType::Text), "text");
    }

    #[test]
    fn test_strategy_type_bitand() {
        // BitAnd with matching bit returns the value
        assert_eq!(StrategyType::Lines & 1u8, 1u8);
        assert_eq!(StrategyType::LinesStrict & 2u8, 2u8);
        assert_eq!(StrategyType::Text & 4u8, 4u8);
        // BitAnd with non-matching bit returns 0
        assert_eq!(StrategyType::Lines & 0u8, 0u8);
        assert_eq!(StrategyType::Lines & StrategyType::LinesStrict, 0u8);
        assert_eq!(StrategyType::Lines & StrategyType::Text, 0u8);
        // BitAnd with combined flags
        assert_eq!(StrategyType::Lines & 3u8, 1u8); // 3 = Lines | LinesStrict
        assert_eq!(StrategyType::Text & 7u8, 4u8); // 7 = Lines | LinesStrict | Text
    }

    // WordsExtractSettings tests
    #[test]
    fn test_words_extract_settings_default() {
        let settings = WordsExtractSettings::default();
        assert_eq!(settings.x_tolerance.into_inner(), 3.0);
        assert_eq!(settings.y_tolerance.into_inner(), 3.0);
        assert!(!settings.keep_blank_chars);
        assert!(!settings.use_text_flow);
        assert!(settings.text_read_in_clockwise);
        assert!(settings.split_at_punctuation.is_none());
        assert!(settings.expand_ligatures);
    }

    #[test]
    fn test_split_punctuation_to_str() {
        let mut settings = WordsExtractSettings::default();

        settings.split_at_punctuation = None;
        assert_eq!(settings.split_punctuation_to_str(), None);

        settings.split_at_punctuation = Some(SplitPunctuation::All);
        assert_eq!(settings.split_punctuation_to_str(), Some("all".to_string()));

        settings.split_at_punctuation = Some(SplitPunctuation::Custom(".,;".to_string()));
        assert_eq!(settings.split_punctuation_to_str(), Some(".,;".to_string()));
    }

    #[test]
    fn test_str_to_split_punctuation() {
        assert!(WordsExtractSettings::str_to_split_punctuation(None).is_none());

        match WordsExtractSettings::str_to_split_punctuation(Some("all")) {
            Some(SplitPunctuation::All) => {}
            _ => panic!("Expected SplitPunctuation::All"),
        }

        match WordsExtractSettings::str_to_split_punctuation(Some(".,;")) {
            Some(SplitPunctuation::Custom(s)) => assert_eq!(s, ".,;"),
            _ => panic!("Expected SplitPunctuation::Custom"),
        }
    }

    #[test]
    fn test_tf_settings_with_text_settings() {
        let settings = TfSettings::default();
        // text_settings should use the same default WordsExtractSettings
        assert_eq!(
            settings.text_settings.x_tolerance,
            WordsExtractSettings::default().x_tolerance
        );
        assert_eq!(
            settings.text_settings.y_tolerance,
            WordsExtractSettings::default().y_tolerance
        );
    }

    // NonNegativeF32 additional tests
    #[test]
    fn test_non_negative_f32_zero_is_valid() {
        let zero = NonNegativeF32::new(0.0, "field");
        assert!(zero.is_ok());
        assert_eq!(zero.unwrap().into_inner(), 0.0);
    }

    #[test]
    fn test_non_negative_f32_positive_is_valid() {
        let positive = NonNegativeF32::new(100.5, "field");
        assert!(positive.is_ok());
        assert_eq!(positive.unwrap().into_inner(), 100.5);
    }

    #[test]
    fn test_non_negative_f32_negative_is_invalid() {
        let negative = NonNegativeF32::new(-0.001, "my_field");
        assert!(negative.is_err());
        let err = negative.unwrap_err();
        assert_eq!(err.field_name, "my_field");
        assert!(err.value < 0.0);
    }

    #[test]
    fn test_non_negative_f32_error_message() {
        let err = NegativeValueError::new("test_field", -5.0);
        let msg = err.to_string();
        assert!(msg.contains("test_field"));
        assert!(msg.contains("-5"));
        assert!(msg.contains("non-negative"));
    }

    #[test]
    fn test_non_negative_f32_deref() {
        let val = NonNegativeF32::new_unchecked(3.0);
        // Test Deref to OrderedFloat<f32>
        let ordered: OrderedFloat<f32> = *val;
        assert_eq!(ordered, OrderedFloat(3.0));
    }

    #[test]
    fn test_non_negative_f32_comparison() {
        let a = NonNegativeF32::new_unchecked(3.0);
        let b = NonNegativeF32::new_unchecked(5.0);
        let c = NonNegativeF32::new_unchecked(3.0);

        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, c);
        assert_ne!(a, b);
    }

    #[test]
    fn test_non_negative_f32_new_unchecked() {
        // new_unchecked should work for valid values
        let val = NonNegativeF32::new_unchecked(10.0);
        assert_eq!(val.into_inner(), 10.0);
    }

    // TfSettings validation tests
    #[test]
    fn test_tf_settings_valid_custom_values() {
        let mut settings = TfSettings::default();
        settings.snap_x_tolerance = NonNegativeF32::new(5.0, "snap_x_tolerance").unwrap();
        settings.snap_y_tolerance = NonNegativeF32::new(10.0, "snap_y_tolerance").unwrap();
        settings.edge_min_length = NonNegativeF32::new(0.0, "edge_min_length").unwrap();

        assert_eq!(settings.snap_x_tolerance.into_inner(), 5.0);
        assert_eq!(settings.snap_y_tolerance.into_inner(), 10.0);
        assert_eq!(settings.edge_min_length.into_inner(), 0.0);
    }

    #[test]
    fn test_tf_settings_negative_snap_x_tolerance_fails() {
        let result = NonNegativeF32::new(-1.0, "snap_x_tolerance");
        assert!(result.is_err());
    }

    #[test]
    fn test_tf_settings_negative_join_tolerance_fails() {
        let result = NonNegativeF32::new(-0.5, "join_x_tolerance");
        assert!(result.is_err());
    }

    #[test]
    fn test_tf_settings_negative_edge_min_length_fails() {
        let result = NonNegativeF32::new(-10.0, "edge_min_length");
        assert!(result.is_err());
    }

    #[test]
    fn test_tf_settings_negative_intersection_tolerance_fails() {
        let result = NonNegativeF32::new(-2.0, "intersection_x_tolerance");
        assert!(result.is_err());
    }

    // WordsExtractSettings validation tests
    #[test]
    fn test_words_extract_settings_valid_custom_values() {
        let mut settings = WordsExtractSettings::default();
        settings.x_tolerance = NonNegativeF32::new(5.0, "x_tolerance").unwrap();
        settings.y_tolerance = NonNegativeF32::new(0.0, "y_tolerance").unwrap();

        assert_eq!(settings.x_tolerance.into_inner(), 5.0);
        assert_eq!(settings.y_tolerance.into_inner(), 0.0);
    }

    #[test]
    fn test_words_extract_settings_negative_x_tolerance_fails() {
        let result = NonNegativeF32::new(-1.0, "x_tolerance");
        assert!(result.is_err());
    }

    #[test]
    fn test_words_extract_settings_negative_y_tolerance_fails() {
        let result = NonNegativeF32::new(-0.1, "y_tolerance");
        assert!(result.is_err());
    }
}
