use crate::clusters::cluster_objects;
use crate::objects::*;
use crate::settings::*;
use itertools::Itertools;
use ordered_float::OrderedFloat;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

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

static PUNCTUATIONS: LazyLock<HashSet<char>> =
    LazyLock::new(|| "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~".chars().collect());

#[derive(Debug, Clone)]
pub(crate) struct Word {
    pub text: String,
    pub bbox: BboxKey,
    pub rotation_degrees: OrderedFloat<f32>,
    pub upright: bool,
}

impl Word {
    pub fn direction(&self) -> &'static str {
        let rotation = self.rotation_degrees.into_inner();
        if self.upright {
            if (rotation - 0.0).abs() < 0.001 {
                "ltr"
            } else {
                "rtl"
            }
        } else {
            // 垂直文本
            if (rotation - 90.0).abs() < 0.001 {
                "ttb"
            } else {
                "btt"
            }
        }
    }
}

impl HasBbox for Word {
    fn bbox(&self) -> BboxKey {
        self.bbox.clone()
    }
}

pub(crate) struct WordMap {
    pub items: Vec<(Word, Vec<Char>)>,
}

impl WordMap {
    pub fn new(items: Vec<(Word, Vec<Char>)>) -> Self {
        Self { items }
    }
}

pub(crate) struct WordExtractor {
    x_tolerance: OrderedFloat<f32>,
    y_tolerance: OrderedFloat<f32>,
    keep_blank_chars: bool,
    use_text_flow: bool,
    horizontal_ltr: bool,
    vertical_ttb: bool,
    split_at_punctuation: HashSet<char>,
    expansions: HashMap<&'static str, &'static str>,
}

impl WordExtractor {
    pub fn new(word_extract_settings: &WordsExtractSettings) -> Self {
        let split_chars = match &word_extract_settings.split_at_punctuation {
            Some(SplitPunctuation::All) => PUNCTUATIONS.clone(),
            Some(SplitPunctuation::Custom(chars)) => chars.chars().collect(),
            None => HashSet::new(),
        };

        Self {
            x_tolerance: word_extract_settings.x_tolerance,
            y_tolerance: word_extract_settings.y_tolerance,
            keep_blank_chars: word_extract_settings.keep_blank_chars,
            use_text_flow: word_extract_settings.use_text_flow,
            horizontal_ltr: word_extract_settings.horizontal_ltr,
            vertical_ttb: word_extract_settings.vertical_ttb,
            split_at_punctuation: split_chars,
            expansions: if word_extract_settings.expand_ligatures {
                LIGATURES.clone()
            } else {
                HashMap::new()
            },
        }
    }
    pub fn merge_chars(&self, ordered_chars: &[Char]) -> Word {
        let (x1, y1, x2, y2) = get_objects_bbox(ordered_chars).unwrap();
        let first_char = &ordered_chars[0];
        let upright = first_char.upright;

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
            upright,
            rotation_degrees: rotation,
        }
    }

    pub fn char_begins_new_word(&self, prev_char: &Char, curr_char: &Char) -> bool {
        let (x, y, ax, bx, cx, ay, cy);

        if curr_char.upright {
            x = self.x_tolerance;
            y = self.y_tolerance;
            ay = prev_char.bbox.1;
            cy = curr_char.bbox.1;

            if self.horizontal_ltr {
                ax = prev_char.bbox.0;
                bx = prev_char.bbox.2;
                cx = curr_char.bbox.0;
            } else {
                ax = -prev_char.bbox.2;
                bx = -prev_char.bbox.0;
                cx = -curr_char.bbox.2;
            }
        } else {
            x = self.y_tolerance;
            y = self.x_tolerance;
            ay = prev_char.bbox.0;
            cy = curr_char.bbox.0;

            if self.vertical_ttb {
                ax = prev_char.bbox.1;
                bx = prev_char.bbox.3;
                cx = curr_char.bbox.1;
            } else {
                ax = -prev_char.bbox.3;
                bx = -prev_char.bbox.1;
                cx = -curr_char.bbox.3;
            }
        }

        (cx < ax) || (cx > bx + x) || (cy > ay + y)
    }

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
    pub fn iter_sort_chars(&self, chars: &[Char]) -> Vec<Char> {
        let mut result = Vec::with_capacity(chars.len());
        let rotation_degrees_key = |char: &Char| char.rotation_degrees;

        let rotation_clusters = cluster_objects(&chars, rotation_degrees_key, OrderedFloat(0.001));

        for rotation_cluster in rotation_clusters {
            if rotation_cluster.is_empty() {
                continue;
            }
            let upright = rotation_cluster[0].upright;
            let sub_key = match upright {
                true => |char: &Char| char.bbox.1,
                false => |char: &Char| char.bbox.0,
            };
            let sub_clusters = cluster_objects(&rotation_cluster, sub_key, self.y_tolerance);

            for mut sc in sub_clusters {
                if upright {
                    // horizontal
                    if self.horizontal_ltr {
                        sc.sort_by(|a, b| a.bbox.0.partial_cmp(&b.bbox.0).unwrap());
                    } else {
                        sc.sort_by(|a, b| b.bbox.0.partial_cmp(&a.bbox.0).unwrap());
                    }
                } else {
                    // vertical
                    if self.vertical_ttb {
                        sc.sort_by(|a, b| a.bbox.1.partial_cmp(&b.bbox.1).unwrap());
                    } else {
                        sc.sort_by(|a, b| b.bbox.1.partial_cmp(&a.bbox.1).unwrap());
                    }
                }
                result.extend(sc);
            }
        }

        result
    }

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

    pub fn extract_wordmap(&self, chars: &[Char]) -> WordMap {
        WordMap::new(self.iter_extract_tuples(chars))
    }

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
    use pdfium_render::prelude::Pdfium;

    fn load_pdfium() -> Pdfium {
        let project_root = env!("CARGO_MANIFEST_DIR");

        #[cfg(target_os = "windows")]
        let pdfium = Pdfium::new(
            Pdfium::bind_to_library(&format!("{}/python/tablers/pdfium.dll", project_root))
                .unwrap(),
        );
        #[cfg(target_os = "macos")]
        let pdfium = Pdfium::new(
            Pdfium::bind_to_library(&format!("{}/python/tablers/libpdfium.dylib", project_root))
                .unwrap(),
        );
        #[cfg(target_os = "linux")]
        let pdfium = Pdfium::new(
            Pdfium::bind_to_library(&format!("{}/python/tablers/libpdfium.so", project_root))
                .unwrap(),
        );

        pdfium
    }

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
            vertical_ttb: false,
            ..Default::default()
        };
        let extractor = WordExtractor::new(&settings);
        let words = extractor.extract_words(chars);

        let horizontal_words: Vec<&Word> = words.iter().filter(|w| w.upright).collect();
        assert_eq!(horizontal_words[0].text, "Agaaaaa:");
        assert_eq!(horizontal_words[0].direction(), "ltr");

        // keep_blank_chars=true
        let settings_with_spaces = WordsExtractSettings {
            vertical_ttb: false,
            keep_blank_chars: true,
            ..Default::default()
        };
        let extractor_with_spaces = WordExtractor::new(&settings_with_spaces);
        let words_w_spaces = extractor_with_spaces.extract_words(chars);

        let horizontal_words_w_spaces: Vec<&Word> =
            words_w_spaces.iter().filter(|w| w.upright).collect();
        assert_eq!(horizontal_words_w_spaces[0].text, "Agaaaaa: AAAA");

        let vertical: Vec<&Word> = words.iter().filter(|w| !w.upright).collect();
        assert_eq!(vertical[0].text, "Aaaaaabag8");
        assert_eq!(vertical[0].direction(), "btt");

        let settings_rtl = WordsExtractSettings {
            horizontal_ltr: false,
            ..Default::default()
        };
        let extractor_rtl = WordExtractor::new(&settings_rtl);
        let words_rtl = extractor_rtl.extract_words(chars);

        let horizontal_rtl: Vec<&Word> = words_rtl.iter().filter(|w| w.upright).collect();
        assert_eq!(horizontal_rtl[1].text, "baaabaaA/AAA");
        assert_eq!(horizontal_rtl[1].direction(), "ltr");
    }
}
