use crate::clusters::cluster_objects;
use crate::objects::*;
use crate::settings::*;
use itertools::Itertools;
use ordered_float::OrderedFloat;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/// Mapping of Unicode ligature characters to their expanded forms.
static LIGATURES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    [
        ("ﬀ", "ff"),
        ("ﬃ", "ffi"),
        ("ﬄ", "ffl"),
        ("ﬁ", "fi"),
        ("ﬂ", "fl"),
        ("ﬆ", "st"),
        ("ﬅ", "st"),
    ]
    .into_iter()
    .collect()
});

/// Set of standard ASCII punctuation characters.
static PUNCTUATIONS: LazyLock<HashSet<char>> =
    LazyLock::new(|| "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~".chars().collect());

/// Represents a word extracted from PDF text.
///
/// A word is a sequence of characters grouped by proximity and alignment.
#[derive(Debug, Clone)]
pub(crate) struct Word {
    /// The text content of the word.
    pub text: String,
    /// The bounding box of the word.
    pub bbox: BboxKey,
    /// The rotation of the word in degrees.
    #[allow(dead_code)]
    pub rotation_degrees: OrderedFloat<f32>,
}

impl HasBbox for Word {
    fn bbox(&self) -> BboxKey {
        self.bbox
    }
}

/// Extracts words from PDF character data.
///
/// This struct handles grouping characters into words based on
/// proximity, alignment, and various configuration options.
pub(crate) struct WordExtractor {
    /// X-axis tolerance for character grouping.
    x_tolerance: OrderedFloat<f32>,
    /// Y-axis tolerance for line grouping.
    y_tolerance: OrderedFloat<f32>,
    /// Whether to preserve whitespace characters.
    keep_blank_chars: bool,
    /// Whether to use PDF text flow order.
    use_text_flow: bool,
    /// Whether text reads in clockwise direction.
    text_read_in_clockwise: bool,
    /// Characters that trigger word splits.
    split_at_punctuation: HashSet<char>,
    /// Ligature expansion mappings.
    expansions: HashMap<&'static str, &'static str>,
}

impl WordExtractor {
    /// Creates a new WordExtractor with the specified settings.
    ///
    /// # Arguments
    ///
    /// * `word_extract_settings` - The settings for word extraction.
    ///
    /// # Returns
    ///
    /// A new WordExtractor instance.
    pub fn new(word_extract_settings: &WordsExtractSettings) -> Self {
        let split_chars = match &word_extract_settings.split_at_punctuation {
            Some(SplitPunctuation::All) => PUNCTUATIONS.clone(),
            Some(SplitPunctuation::Custom(chars)) => chars.chars().collect(),
            None => HashSet::new(),
        };

        Self {
            x_tolerance: *word_extract_settings.x_tolerance,
            y_tolerance: *word_extract_settings.y_tolerance,
            keep_blank_chars: word_extract_settings.keep_blank_chars,
            use_text_flow: word_extract_settings.use_text_flow,
            text_read_in_clockwise: word_extract_settings.text_read_in_clockwise,
            split_at_punctuation: split_chars,
            expansions: if word_extract_settings.expand_ligatures {
                LIGATURES.clone()
            } else {
                HashMap::new()
            },
        }
    }
    /// Merges a sequence of characters into a single word.
    ///
    /// # Arguments
    ///
    /// * `ordered_chars` - The characters to merge (must be non-empty).
    ///
    /// # Returns
    ///
    /// A Word containing the merged text and combined bounding box.
    pub fn merge_chars(&self, ordered_chars: &[Char]) -> Word {
        let (x1, y1, x2, y2) = get_objects_bbox(ordered_chars).unwrap();
        let first_char = &ordered_chars[0];

        let rotation = first_char.rotation_degrees;
        let chars_iter: Box<dyn Iterator<Item = &Char>> =
            if (OrderedFloat(270.0f32) - rotation).abs() < 0.001 {
                Box::new(ordered_chars.iter().rev())
            } else {
                Box::new(ordered_chars.iter())
            };

        let text: String = chars_iter
            .map(|c| {
                let unicode_char = c.unicode_char.as_ref().unwrap();
                self.expansions
                    .get(unicode_char.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| unicode_char.clone())
            })
            .collect();

        Word {
            text,
            bbox: (x1, y1, x2, y2),
            rotation_degrees: rotation,
        }
    }

    /// Determines if a character should start a new word.
    ///
    /// Based on the position and rotation of the current character
    /// relative to the previous character.
    ///
    /// # Arguments
    ///
    /// * `prev_char` - The previous character.
    /// * `curr_char` - The current character.
    ///
    /// # Returns
    ///
    /// `true` if the current character should start a new word.
    pub fn char_begins_new_word(&self, prev_char: &Char, curr_char: &Char) -> bool {
        let (x, y, ax, bx, cx, ay, cy);

        if (curr_char.rotation_degrees >= OrderedFloat(-0.001f32)
            && curr_char.rotation_degrees < OrderedFloat(45.0f32))
            || (curr_char.rotation_degrees >= OrderedFloat(315.0f32)
                && curr_char.rotation_degrees < OrderedFloat(360.001f32))
        {
            x = self.x_tolerance;
            y = self.y_tolerance;
            ay = prev_char.bbox.1;
            cy = curr_char.bbox.1;

            if self.text_read_in_clockwise {
                ax = prev_char.bbox.0;
                bx = prev_char.bbox.2;
                cx = curr_char.bbox.0;
            } else {
                ax = -prev_char.bbox.2;
                bx = -prev_char.bbox.0;
                cx = -curr_char.bbox.2;
            }
        } else if curr_char.rotation_degrees >= OrderedFloat(45.0f32)
            && curr_char.rotation_degrees < OrderedFloat(135.0f32)
        {
            x = self.y_tolerance;
            y = self.x_tolerance;
            ay = prev_char.bbox.0;
            cy = curr_char.bbox.0;

            if self.text_read_in_clockwise {
                ax = prev_char.bbox.1;
                bx = prev_char.bbox.3;
                cx = curr_char.bbox.1;
            } else {
                ax = -prev_char.bbox.3;
                bx = -prev_char.bbox.1;
                cx = -curr_char.bbox.3;
            }
        } else if curr_char.rotation_degrees >= OrderedFloat(135.0f32)
            && curr_char.rotation_degrees < OrderedFloat(225.0f32)
        {
            x = self.x_tolerance;
            y = self.y_tolerance;
            ay = prev_char.bbox.3;
            cy = curr_char.bbox.3;

            if self.text_read_in_clockwise {
                ax = -prev_char.bbox.2;
                bx = -prev_char.bbox.0;
                cx = -curr_char.bbox.2;
            } else {
                ax = prev_char.bbox.0;
                bx = prev_char.bbox.2;
                cx = curr_char.bbox.0;
            }
        } else {
            x = self.y_tolerance;
            y = self.x_tolerance;
            ay = prev_char.bbox.0;
            cy = curr_char.bbox.0;

            if self.text_read_in_clockwise {
                ax = -prev_char.bbox.3;
                bx = -prev_char.bbox.1;
                cx = -curr_char.bbox.3;
            } else {
                ax = prev_char.bbox.1;
                bx = prev_char.bbox.3;
                cx = curr_char.bbox.1;
            }
        }

        (cx < ax) || (cx > bx + x) || (cy > ay + y)
    }

    /// Groups ordered characters into word groups.
    ///
    /// # Arguments
    ///
    /// * `ordered_chars` - Characters in reading order.
    ///
    /// # Returns
    ///
    /// A vector where each element is a group of characters forming a word.
    pub fn iter_chars_to_words(&self, ordered_chars: Vec<Char>) -> Vec<Vec<Char>> {
        let mut words = Vec::new();
        let mut current_word: Vec<Char> = Vec::new();

        for char in ordered_chars {
            let text = &char.unicode_char.as_ref().unwrap();

            if !self.keep_blank_chars && text.chars().all(|c| c.is_whitespace()) {
                if !current_word.is_empty() {
                    words.push(std::mem::take(&mut current_word));
                }
            } else if text.len() == 1
                && self
                    .split_at_punctuation
                    .contains(&text.chars().next().unwrap())
            {
                if !current_word.is_empty() {
                    words.push(std::mem::take(&mut current_word));
                }
                words.push(vec![char.clone()]);
            } else if !current_word.is_empty()
                && self.char_begins_new_word(current_word.last().unwrap(), &char)
            {
                words.push(std::mem::take(&mut current_word));
                current_word.push(char.clone());
            } else {
                current_word.push(char.clone());
            }
        }

        if !current_word.is_empty() {
            words.push(current_word);
        }

        words
    }
    /// Sorts characters into reading order.
    ///
    /// Characters are first clustered by rotation, then sorted within
    /// each cluster based on position.
    ///
    /// # Arguments
    ///
    /// * `chars` - The characters to sort.
    ///
    /// # Returns
    ///
    /// Characters sorted in reading order.
    pub fn iter_sort_chars(&self, chars: &[Char]) -> Vec<Char> {
        let mut result = Vec::with_capacity(chars.len());
        let rotation_degrees_key = |char: &Char| char.rotation_degrees;

        let rotation_clusters = cluster_objects(chars, rotation_degrees_key, OrderedFloat(0.001));

        for rotation_cluster in rotation_clusters {
            if rotation_cluster.is_empty() {
                continue;
            }
            let rotation_degrees = rotation_cluster[0].rotation_degrees;
            let upright = rotation_cluster[0].upright;
            let sub_key = match upright {
                true => |char: &Char| char.bbox.1,
                false => |char: &Char| char.bbox.0,
            };
            let sub_clusters = cluster_objects(&rotation_cluster, sub_key, self.y_tolerance);

            for mut sc in sub_clusters {
                if (rotation_degrees >= OrderedFloat(-0.001f32)
                    && rotation_degrees < OrderedFloat(45.0f32))
                    || (rotation_degrees >= OrderedFloat(315.0f32)
                        && rotation_degrees < OrderedFloat(360.001f32))
                {
                    if self.text_read_in_clockwise {
                        sc.sort_by(|a, b| a.bbox.0.partial_cmp(&b.bbox.0).unwrap());
                    } else {
                        sc.sort_by(|a, b| b.bbox.0.partial_cmp(&a.bbox.0).unwrap());
                    }
                } else if rotation_degrees >= OrderedFloat(45.0f32)
                    && rotation_degrees < OrderedFloat(135.0f32)
                {
                    if self.text_read_in_clockwise {
                        sc.sort_by(|a, b| a.bbox.1.partial_cmp(&b.bbox.1).unwrap());
                    } else {
                        sc.sort_by(|a, b| b.bbox.1.partial_cmp(&a.bbox.1).unwrap());
                    }
                } else if rotation_degrees >= OrderedFloat(135.0f32)
                    && rotation_degrees < OrderedFloat(225.0f32)
                {
                    if self.text_read_in_clockwise {
                        sc.sort_by(|a, b| b.bbox.0.partial_cmp(&a.bbox.0).unwrap());
                    } else {
                        sc.sort_by(|a, b| a.bbox.0.partial_cmp(&b.bbox.0).unwrap());
                    }
                } else if self.text_read_in_clockwise {
                    sc.sort_by(|a, b| b.bbox.1.partial_cmp(&a.bbox.1).unwrap());
                } else {
                    sc.sort_by(|a, b| a.bbox.1.partial_cmp(&b.bbox.1).unwrap());
                }
                result.extend(sc);
            }
        }

        result
    }

    /// Extracts words along with their source characters.
    ///
    /// # Arguments
    ///
    /// * `chars` - The characters to process.
    ///
    /// # Returns
    ///
    /// A vector of tuples containing each word and its source characters.
    pub fn iter_extract_tuples(&self, chars: &[Char]) -> Vec<(Word, Vec<Char>)> {
        let ordered_chars = if self.use_text_flow {
            chars.to_vec()
        } else {
            self.iter_sort_chars(chars)
        };

        let char_groups: Vec<Vec<Char>> = ordered_chars
            .into_iter()
            .chunk_by(|c| c.rotation_degrees)
            .into_iter()
            .map(|(_, group)| group.collect())
            .collect();

        let mut result = Vec::new();
        for char_group in char_groups {
            let word_groups = self.iter_chars_to_words(char_group);
            for word_chars in word_groups {
                let word = self.merge_chars(&word_chars);
                result.push((word, word_chars));
            }
        }

        result
    }

    /// Extracts words from a sequence of characters.
    ///
    /// This is the main entry point for word extraction.
    ///
    /// # Arguments
    ///
    /// * `chars` - The characters to process.
    ///
    /// # Returns
    ///
    /// A vector of extracted words.
    pub fn extract_words(&self, chars: &[Char]) -> Vec<Word> {
        self.iter_extract_tuples(chars)
            .into_iter()
            .map(|(word, _)| word)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::Page;
    use crate::test_utils::load_pdfium;

    #[test]
    fn test_extract_words() {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let pdfium = load_pdfium();

        let pdf_path = format!("{}/tests/data/words-extract.pdf", project_root);
        let doc = pdfium.load_pdf_from_file(&pdf_path, None).unwrap();
        let page = doc.pages().get(0).unwrap();
        let pdf_page = Page::new(unsafe { std::mem::transmute(page) }, 0);

        let objects = pdf_page.objects.borrow();
        let chars = &objects.as_ref().unwrap().chars;

        let settings = WordsExtractSettings {
            ..Default::default()
        };
        let extractor = WordExtractor::new(&settings);
        let words = extractor.extract_words(chars);

        let horizontal_words: Vec<&Word> = words
            .iter()
            .filter(|w| (w.rotation_degrees - 0.0).abs() < 0.001)
            .collect();
        assert_eq!(horizontal_words[0].text, "Agaaaaa:");

        // keep_blank_chars=true
        let settings_with_spaces = WordsExtractSettings {
            keep_blank_chars: true,
            ..Default::default()
        };
        let extractor_with_spaces = WordExtractor::new(&settings_with_spaces);
        let words_w_spaces = extractor_with_spaces.extract_words(chars);

        let horizontal_words_w_spaces: Vec<&Word> = words_w_spaces
            .iter()
            .filter(|w| (w.rotation_degrees - 0.0).abs() < 0.001)
            .collect();
        assert_eq!(horizontal_words_w_spaces[0].text, "Agaaaaa: AAAA");

        let vertical: Vec<&Word> = words
            .iter()
            .filter(|w| (w.rotation_degrees - 0.0).abs() > 45.0)
            .collect();
        assert_eq!(vertical[0].text, "Aaaaaabag8");

        let settings_rtl = WordsExtractSettings {
            text_read_in_clockwise: false,
            ..Default::default()
        };
        let extractor_rtl = WordExtractor::new(&settings_rtl);
        let words_rtl = extractor_rtl.extract_words(chars);

        let horizontal_rtl: Vec<&Word> = words_rtl
            .iter()
            .filter(|w| (w.rotation_degrees - 0.0).abs() < 0.001)
            .collect();
        assert_eq!(horizontal_rtl[1].text, "baaabaaA/AAA");
    }
}
