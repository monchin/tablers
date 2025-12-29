"""
Tests for TfSettings and WordsExtractSettings classes.
"""

import pytest
from tablers import TfSettings, WordsExtractSettings


class TestWordsExtractSettings:
    """Tests for WordsExtractSettings class."""

    def test_default_initialization(self) -> None:
        """WordsExtractSettings can be initialized with default values."""
        settings = WordsExtractSettings()
        assert settings is not None

    def test_x_tolerance_attribute(self) -> None:
        """x_tolerance attribute should be accessible."""
        settings = WordsExtractSettings()
        assert isinstance(settings.x_tolerance, float)
        assert settings.x_tolerance >= 0

    def test_y_tolerance_attribute(self) -> None:
        """y_tolerance attribute should be accessible."""
        settings = WordsExtractSettings()
        assert isinstance(settings.y_tolerance, float)
        assert settings.y_tolerance >= 0

    def test_keep_blank_chars_attribute(self) -> None:
        """keep_blank_chars attribute should be accessible."""
        settings = WordsExtractSettings()
        assert isinstance(settings.keep_blank_chars, bool)

    def test_use_text_flow_attribute(self) -> None:
        """use_text_flow attribute should be accessible."""
        settings = WordsExtractSettings()
        assert isinstance(settings.use_text_flow, bool)

    def test_text_read_in_clockwise_attribute(self) -> None:
        """text_read_in_clockwise attribute should be accessible."""
        settings = WordsExtractSettings()
        assert isinstance(settings.text_read_in_clockwise, bool)

    def test_split_at_punctuation_attribute(self) -> None:
        """split_at_punctuation attribute should be accessible."""
        settings = WordsExtractSettings()
        # Can be str or None
        assert settings.split_at_punctuation is None or isinstance(
            settings.split_at_punctuation, str
        )

    def test_expand_ligatures_attribute(self) -> None:
        """expand_ligatures attribute should be accessible."""
        settings = WordsExtractSettings()
        assert isinstance(settings.expand_ligatures, bool)

    def test_custom_x_tolerance(self) -> None:
        """WordsExtractSettings can be initialized with custom x_tolerance."""
        settings = WordsExtractSettings(x_tolerance=5.0)
        assert settings.x_tolerance == 5.0

    def test_custom_y_tolerance(self) -> None:
        """WordsExtractSettings can be initialized with custom y_tolerance."""
        settings = WordsExtractSettings(y_tolerance=10.0)
        assert settings.y_tolerance == 10.0

    def test_custom_keep_blank_chars(self) -> None:
        """WordsExtractSettings can be initialized with custom keep_blank_chars."""
        settings = WordsExtractSettings(keep_blank_chars=True)
        assert settings.keep_blank_chars is True

    def test_custom_split_at_punctuation_all(self) -> None:
        """WordsExtractSettings can use 'all' for split_at_punctuation."""
        settings = WordsExtractSettings(split_at_punctuation="all")
        assert settings.split_at_punctuation == "all"

    def test_custom_split_at_punctuation_custom(self) -> None:
        """WordsExtractSettings can use custom punctuation string."""
        settings = WordsExtractSettings(split_at_punctuation=".,;")
        assert settings.split_at_punctuation == ".,;"

    def test_repr(self) -> None:
        """WordsExtractSettings should have a string representation."""
        settings = WordsExtractSettings()
        repr_str = repr(settings)
        assert isinstance(repr_str, str)
        assert len(repr_str) > 0

    def test_equality(self) -> None:
        """Two WordsExtractSettings with same values should be equal."""
        settings1 = WordsExtractSettings()
        settings2 = WordsExtractSettings()
        assert settings1 == settings2

    def test_inequality_with_different_values(self) -> None:
        """Two WordsExtractSettings with different values should not be equal."""
        settings1 = WordsExtractSettings(x_tolerance=3.0)
        settings2 = WordsExtractSettings(x_tolerance=5.0)
        assert settings1 != settings2


class TestTfSettings:
    """Tests for TfSettings class."""

    def test_default_initialization(self) -> None:
        """TfSettings can be initialized with default values."""
        settings = TfSettings()
        assert settings is not None

    def test_vertical_strategy_attribute(self) -> None:
        """vertical_strategy attribute should be accessible."""
        settings = TfSettings()
        assert settings.vertical_strategy in ("lines", "lines_strict", "text")

    def test_horizontal_strategy_attribute(self) -> None:
        """horizontal_strategy attribute should be accessible."""
        settings = TfSettings()
        assert settings.horizontal_strategy in ("lines", "lines_strict", "text")

    def test_snap_tolerances(self) -> None:
        """snap_x_tolerance and snap_y_tolerance should be accessible."""
        settings = TfSettings()
        assert isinstance(settings.snap_x_tolerance, float)
        assert isinstance(settings.snap_y_tolerance, float)
        assert settings.snap_x_tolerance >= 0
        assert settings.snap_y_tolerance >= 0

    def test_join_tolerances(self) -> None:
        """join_x_tolerance and join_y_tolerance should be accessible."""
        settings = TfSettings()
        assert isinstance(settings.join_x_tolerance, float)
        assert isinstance(settings.join_y_tolerance, float)
        assert settings.join_x_tolerance >= 0
        assert settings.join_y_tolerance >= 0

    def test_edge_min_length(self) -> None:
        """edge_min_length attribute should be accessible."""
        settings = TfSettings()
        assert isinstance(settings.edge_min_length, float)
        assert settings.edge_min_length >= 0

    def test_edge_min_length_prefilter(self) -> None:
        """edge_min_length_prefilter attribute should be accessible."""
        settings = TfSettings()
        assert isinstance(settings.edge_min_length_prefilter, float)
        assert settings.edge_min_length_prefilter >= 0

    def test_min_words_attributes(self) -> None:
        """min_words_vertical and min_words_horizontal should be accessible."""
        settings = TfSettings()
        assert isinstance(settings.min_words_vertical, int)
        assert isinstance(settings.min_words_horizontal, int)
        assert settings.min_words_vertical >= 0
        assert settings.min_words_horizontal >= 0

    def test_intersection_tolerances(self) -> None:
        """intersection tolerances should be accessible."""
        settings = TfSettings()
        assert isinstance(settings.intersection_x_tolerance, float)
        assert isinstance(settings.intersection_y_tolerance, float)
        assert settings.intersection_x_tolerance >= 0
        assert settings.intersection_y_tolerance >= 0

    def test_text_settings_attribute(self) -> None:
        """text_settings attribute should return WordsExtractSettings."""
        settings = TfSettings()
        assert isinstance(settings.text_settings, WordsExtractSettings)

    def test_text_tolerances(self) -> None:
        """text_x_tolerance and text_y_tolerance should be accessible."""
        settings = TfSettings()
        assert isinstance(settings.text_x_tolerance, float)
        assert isinstance(settings.text_y_tolerance, float)
        assert settings.text_x_tolerance >= 0
        assert settings.text_y_tolerance >= 0

    def test_custom_vertical_strategy(self) -> None:
        """TfSettings can be initialized with custom vertical_strategy."""
        settings = TfSettings(vertical_strategy="text")
        assert settings.vertical_strategy == "text"

    def test_custom_horizontal_strategy(self) -> None:
        """TfSettings can be initialized with custom horizontal_strategy."""
        settings = TfSettings(horizontal_strategy="lines_strict")
        assert settings.horizontal_strategy == "lines_strict"

    def test_custom_snap_tolerances(self) -> None:
        """TfSettings can be initialized with custom snap tolerances."""
        settings = TfSettings(snap_x_tolerance=5.0, snap_y_tolerance=10.0)
        assert settings.snap_x_tolerance == 5.0
        assert settings.snap_y_tolerance == 10.0

    def test_custom_edge_min_length(self) -> None:
        """TfSettings can be initialized with custom edge_min_length."""
        settings = TfSettings(edge_min_length=20.0)
        assert settings.edge_min_length == 20.0

    def test_repr(self) -> None:
        """TfSettings should have a string representation."""
        settings = TfSettings()
        repr_str = repr(settings)
        assert isinstance(repr_str, str)
        assert len(repr_str) > 0

    def test_equality(self) -> None:
        """Two TfSettings with same values should be equal."""
        settings1 = TfSettings()
        settings2 = TfSettings()
        assert settings1 == settings2

    def test_inequality_with_different_values(self) -> None:
        """Two TfSettings with different values should not be equal."""
        settings1 = TfSettings(snap_x_tolerance=3.0)
        settings2 = TfSettings(snap_x_tolerance=5.0)
        assert settings1 != settings2

    def test_multiple_custom_values(self) -> None:
        """TfSettings can be initialized with multiple custom values."""
        settings = TfSettings(
            vertical_strategy="lines_strict",
            horizontal_strategy="text",
            snap_x_tolerance=5.0,
            snap_y_tolerance=5.0,
            edge_min_length=15.0,
        )
        assert settings.vertical_strategy == "lines_strict"
        assert settings.horizontal_strategy == "text"
        assert settings.snap_x_tolerance == 5.0
        assert settings.snap_y_tolerance == 5.0
        assert settings.edge_min_length == 15.0


class TestNonNegativeValidation:
    """Tests for non-negative value validation in settings."""

    # WordsExtractSettings negative value tests
    def test_words_extract_settings_negative_x_tolerance_raises(self) -> None:
        """WordsExtractSettings raises ValueError for negative x_tolerance."""
        with pytest.raises(ValueError, match="x_tolerance.*non-negative"):
            WordsExtractSettings(x_tolerance=-1.0)

    def test_words_extract_settings_negative_y_tolerance_raises(self) -> None:
        """WordsExtractSettings raises ValueError for negative y_tolerance."""
        with pytest.raises(ValueError, match="y_tolerance.*non-negative"):
            WordsExtractSettings(y_tolerance=-0.5)

    def test_words_extract_settings_zero_tolerance_valid(self) -> None:
        """WordsExtractSettings accepts zero for tolerance values."""
        settings = WordsExtractSettings(x_tolerance=0.0, y_tolerance=0.0)
        assert settings.x_tolerance == 0.0
        assert settings.y_tolerance == 0.0

    def test_words_extract_settings_setter_negative_x_tolerance_raises(self) -> None:
        """WordsExtractSettings setter raises ValueError for negative x_tolerance."""
        settings = WordsExtractSettings()
        with pytest.raises(ValueError, match="x_tolerance.*non-negative"):
            settings.x_tolerance = -1.0

    def test_words_extract_settings_setter_negative_y_tolerance_raises(self) -> None:
        """WordsExtractSettings setter raises ValueError for negative y_tolerance."""
        settings = WordsExtractSettings()
        with pytest.raises(ValueError, match="y_tolerance.*non-negative"):
            settings.y_tolerance = -0.1

    # TfSettings negative value tests
    def test_tf_settings_negative_snap_x_tolerance_raises(self) -> None:
        """TfSettings raises ValueError for negative snap_x_tolerance."""
        with pytest.raises(ValueError, match="snap_x_tolerance.*non-negative"):
            TfSettings(snap_x_tolerance=-1.0)

    def test_tf_settings_negative_snap_y_tolerance_raises(self) -> None:
        """TfSettings raises ValueError for negative snap_y_tolerance."""
        with pytest.raises(ValueError, match="snap_y_tolerance.*non-negative"):
            TfSettings(snap_y_tolerance=-2.0)

    def test_tf_settings_negative_join_x_tolerance_raises(self) -> None:
        """TfSettings raises ValueError for negative join_x_tolerance."""
        with pytest.raises(ValueError, match="join_x_tolerance.*non-negative"):
            TfSettings(join_x_tolerance=-0.5)

    def test_tf_settings_negative_join_y_tolerance_raises(self) -> None:
        """TfSettings raises ValueError for negative join_y_tolerance."""
        with pytest.raises(ValueError, match="join_y_tolerance.*non-negative"):
            TfSettings(join_y_tolerance=-3.0)

    def test_tf_settings_negative_edge_min_length_raises(self) -> None:
        """TfSettings raises ValueError for negative edge_min_length."""
        with pytest.raises(ValueError, match="edge_min_length.*non-negative"):
            TfSettings(edge_min_length=-5.0)

    def test_tf_settings_negative_edge_min_length_prefilter_raises(self) -> None:
        """TfSettings raises ValueError for negative edge_min_length_prefilter."""
        with pytest.raises(ValueError, match="edge_min_length_prefilter.*non-negative"):
            TfSettings(edge_min_length_prefilter=-1.0)

    def test_tf_settings_negative_intersection_x_tolerance_raises(self) -> None:
        """TfSettings raises ValueError for negative intersection_x_tolerance."""
        with pytest.raises(ValueError, match="intersection_x_tolerance.*non-negative"):
            TfSettings(intersection_x_tolerance=-1.0)

    def test_tf_settings_negative_intersection_y_tolerance_raises(self) -> None:
        """TfSettings raises ValueError for negative intersection_y_tolerance."""
        with pytest.raises(ValueError, match="intersection_y_tolerance.*non-negative"):
            TfSettings(intersection_y_tolerance=-2.5)

    def test_tf_settings_negative_text_x_tolerance_raises(self) -> None:
        """TfSettings raises ValueError for negative text_x_tolerance."""
        with pytest.raises(ValueError, match="text_x_tolerance.*non-negative"):
            TfSettings(text_x_tolerance=-1.0)

    def test_tf_settings_negative_text_y_tolerance_raises(self) -> None:
        """TfSettings raises ValueError for negative text_y_tolerance."""
        with pytest.raises(ValueError, match="text_y_tolerance.*non-negative"):
            TfSettings(text_y_tolerance=-0.5)

    def test_tf_settings_zero_values_valid(self) -> None:
        """TfSettings accepts zero for all numeric tolerance values."""
        settings = TfSettings(
            snap_x_tolerance=0.0,
            snap_y_tolerance=0.0,
            join_x_tolerance=0.0,
            join_y_tolerance=0.0,
            edge_min_length=0.0,
            edge_min_length_prefilter=0.0,
            intersection_x_tolerance=0.0,
            intersection_y_tolerance=0.0,
            text_x_tolerance=0.0,
            text_y_tolerance=0.0,
        )
        assert settings.snap_x_tolerance == 0.0
        assert settings.snap_y_tolerance == 0.0
        assert settings.join_x_tolerance == 0.0
        assert settings.join_y_tolerance == 0.0
        assert settings.edge_min_length == 0.0
        assert settings.edge_min_length_prefilter == 0.0
        assert settings.intersection_x_tolerance == 0.0
        assert settings.intersection_y_tolerance == 0.0
        assert settings.text_x_tolerance == 0.0
        assert settings.text_y_tolerance == 0.0

    # TfSettings setter tests
    def test_tf_settings_setter_negative_snap_x_tolerance_raises(self) -> None:
        """TfSettings setter raises ValueError for negative snap_x_tolerance."""
        settings = TfSettings()
        with pytest.raises(ValueError, match="snap_x_tolerance.*non-negative"):
            settings.snap_x_tolerance = -1.0

    def test_tf_settings_setter_negative_snap_y_tolerance_raises(self) -> None:
        """TfSettings setter raises ValueError for negative snap_y_tolerance."""
        settings = TfSettings()
        with pytest.raises(ValueError, match="snap_y_tolerance.*non-negative"):
            settings.snap_y_tolerance = -1.0

    def test_tf_settings_setter_negative_join_x_tolerance_raises(self) -> None:
        """TfSettings setter raises ValueError for negative join_x_tolerance."""
        settings = TfSettings()
        with pytest.raises(ValueError, match="join_x_tolerance.*non-negative"):
            settings.join_x_tolerance = -1.0

    def test_tf_settings_setter_negative_join_y_tolerance_raises(self) -> None:
        """TfSettings setter raises ValueError for negative join_y_tolerance."""
        settings = TfSettings()
        with pytest.raises(ValueError, match="join_y_tolerance.*non-negative"):
            settings.join_y_tolerance = -1.0

    def test_tf_settings_setter_negative_edge_min_length_raises(self) -> None:
        """TfSettings setter raises ValueError for negative edge_min_length."""
        settings = TfSettings()
        with pytest.raises(ValueError, match="edge_min_length.*non-negative"):
            settings.edge_min_length = -1.0

    def test_tf_settings_setter_negative_edge_min_length_prefilter_raises(self) -> None:
        """TfSettings setter raises ValueError for negative edge_min_length_prefilter."""
        settings = TfSettings()
        with pytest.raises(ValueError, match="edge_min_length_prefilter.*non-negative"):
            settings.edge_min_length_prefilter = -1.0

    def test_tf_settings_setter_negative_intersection_x_tolerance_raises(self) -> None:
        """TfSettings setter raises ValueError for negative intersection_x_tolerance."""
        settings = TfSettings()
        with pytest.raises(ValueError, match="intersection_x_tolerance.*non-negative"):
            settings.intersection_x_tolerance = -1.0

    def test_tf_settings_setter_negative_intersection_y_tolerance_raises(self) -> None:
        """TfSettings setter raises ValueError for negative intersection_y_tolerance."""
        settings = TfSettings()
        with pytest.raises(ValueError, match="intersection_y_tolerance.*non-negative"):
            settings.intersection_y_tolerance = -1.0

    def test_tf_settings_setter_negative_text_x_tolerance_raises(self) -> None:
        """TfSettings setter raises ValueError for negative text_x_tolerance."""
        settings = TfSettings()
        with pytest.raises(ValueError, match="text_x_tolerance.*non-negative"):
            settings.text_x_tolerance = -1.0

    def test_tf_settings_setter_negative_text_y_tolerance_raises(self) -> None:
        """TfSettings setter raises ValueError for negative text_y_tolerance."""
        settings = TfSettings()
        with pytest.raises(ValueError, match="text_y_tolerance.*non-negative"):
            settings.text_y_tolerance = -1.0
