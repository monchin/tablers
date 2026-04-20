use crate::objects::*;
use ordered_float::OrderedFloat;
use pdfium_render::prelude::PdfPage as PdfiumPage;
use pdfium_render::prelude::{PdfPathFillMode, *};
use std::cell::RefCell;
use std::char::decode_utf16;
use std::cmp;

#[inline]
fn merge_two_bboxes(a: BboxKey, b: BboxKey) -> BboxKey {
    (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3))
}

/// Represents a PDF page with extracted objects.
///
/// This struct wraps a Pdfium page and provides methods to extract
/// and access various objects like characters, lines, and rectangles.
pub struct Page {
    /// The underlying Pdfium page object.
    pub inner: PdfiumPage<'static>,
    /// The zero-based index of this page in the document.
    pub page_idx: usize,
    /// Cached extracted objects from the page.
    pub objects: RefCell<Option<Objects>>,
}

impl Page {
    /// Creates a new Page instance and automatically extracts objects.
    ///
    /// # Arguments
    ///
    /// * `inner` - The Pdfium page object.
    /// * `page_idx` - The zero-based index of the page.
    ///
    /// # Returns
    ///
    /// A new Page instance with extracted objects.
    pub fn new(inner: PdfiumPage<'static>, page_idx: usize) -> Self {
        let page = Self {
            inner,
            page_idx,
            objects: RefCell::new(None),
        };
        page.extract_objects();
        page
    }

    /// Clears the cached objects to free memory.
    pub fn clear(&self) {
        self.objects.replace(None);
    }

    /// Returns the width of the page in points.
    pub fn width(&self) -> f32 {
        self.inner.width().value
    }

    /// Returns the height of the page in points.
    pub fn height(&self) -> f32 {
        self.inner.height().value
    }

    /// Returns the rotation of the page.
    pub fn rotation_degrees(&self) -> PdfPageRenderRotation {
        self.inner.rotation().unwrap()
    }

    /// Extracts objects from the page if not already cached.
    ///
    /// Objects include characters, lines, and rectangles found in the page content.
    pub fn extract_objects(&self) {
        if self.objects.borrow().is_none() {
            let objects = self.extract_objects_from_page();
            self.objects.replace(Some(objects));
        }
    }

    /// Extracts all objects from the page content.
    ///
    /// # Returns
    ///
    /// An Objects struct containing all extracted characters, lines, and rectangles.
    fn extract_objects_from_page(&self) -> Objects {
        let mut objects = Objects {
            rects: vec![],
            lines: vec![],
            chars: vec![],
        };
        self.process_chars(&mut objects);
        for obj in self.inner.objects().iter() {
            if let Some(obj) = obj.as_path_object() {
                self.process_path_obj(&mut objects, obj, None);
            } else if let Some(obj) = obj.as_x_object_form_object() {
                self.process_x_object_form_obj(&mut objects, obj, None);
            }
        }

        objects
    }

    /// Converts a y-coordinate from top-origin to bottom-origin coordinate system.
    ///
    /// PDF uses bottom-left as origin, but for table extraction we use top-left.
    ///
    /// # Arguments
    ///
    /// * `y` - The y-coordinate in top-origin system.
    ///
    /// # Returns
    ///
    /// The converted y-coordinate in bottom-origin system.
    #[inline]
    fn get_v_coord_with_bottom_origin(&self, y: f32) -> OrderedFloat<f32> {
        if self.rotation_degrees() == PdfPageRenderRotation::Degrees90
            || self.rotation_degrees() == PdfPageRenderRotation::Degrees270
        {
            OrderedFloat::from(self.width() - y)
        } else {
            OrderedFloat::from(self.height() - y)
        }
    }

    // fn count_chars_rotation(&self, chars: &[Char]) -> HashMap<u16, usize> {
    //     let mut result = HashMap::new();
    //     for char in chars {
    //         let rotation: u16 = char.rotation_degrees.round() as u16;
    //         let count = result.entry(rotation).or_insert(0);
    //         *count += 1;
    //     }
    //     result
    // }

    // fn deal_with_page_not_rotated_but_most_chars_rotated(&self, objects: &mut Objects) {
    //     let chars = &mut objects.chars;
    //     let n_chars = chars.len();
    //     if n_chars == 0 {
    //         return;
    //     }
    //     let count_res = self.count_chars_rotation(chars);
    //     let count_90 = *count_res.get(&90u16).unwrap_or(&0);
    //     let count_270 = *count_res.get(&270u16).unwrap_or(&0);
    //     if (count_90 as f32 / n_chars as f32 > 0.9) || (count_270 as f32 / n_chars as f32 > 0.9) {
    //         chars.iter_mut().for_each(|char| {
    //             if char.rotation_degrees == 90.0 || char.rotation_degrees == 270.0 {
    //                 char.upright = true;
    //             }
    //         });
    //     }
    //     let max_entry = count_res.iter().max_by_key(|(_, value)| *value);
    //     if let Some((rotation, _)) = max_entry {
    //         *self.most_chars_rotation_degrees.borrow_mut() = *rotation as f32;
    //     }
    // }

    /// Processes and extracts all text characters from the page.
    ///
    /// # Arguments
    ///
    /// * `objects` - The Objects struct to populate with extracted characters.
    fn process_chars(&self, objects: &mut Objects) {
        let text = self.inner.text().unwrap();

        // Text stream indices follow UTF-16 code units per PDFium public/fpdf_text.h (main), e.g.
        // FPDFText_GetText (UCS-2 values per index) and FPDFText_GetBoundedText (UTF-16 values).
        // Supplementary-plane chars use two units—merge surrogates below.
        let mut units: Vec<(Result<u16, std::num::TryFromIntError>, BboxKey, f32)> = Vec::new();
        for character in text.chars().iter() {
            let u = u16::try_from(character.unicode_value());
            units.push((
                u,
                self.bbox_for_text_char(&character),
                character.get_rotation_clockwise_degrees(),
            ));
        }

        let mut i = 0usize;
        while i < units.len() {
            let unit = match units[i].0 {
                Ok(u) => u,
                Err(_) => {
                    let (_, bbox, rotation_degrees) = &units[i];
                    objects.chars.push(Char {
                        unicode_char: None,
                        bbox: *bbox,
                        rotation_degrees: OrderedFloat::from(*rotation_degrees),
                        upright: *rotation_degrees == 0.0 || *rotation_degrees == 180.0,
                    });
                    i += 1;
                    continue;
                }
            };

            let (scalar, consumed) = if (0xD800..=0xDBFF).contains(&unit) {
                match units
                    .get(i + 1)
                    .and_then(|(u, _, _)| u.as_ref().ok().copied())
                {
                    Some(lo) if (0xDC00..=0xDFFF).contains(&lo) => {
                        let c = decode_utf16([unit, lo])
                            .next()
                            .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
                            .unwrap_or(char::REPLACEMENT_CHARACTER);
                        (c, 2usize)
                    }
                    _ => (char::REPLACEMENT_CHARACTER, 1usize),
                }
            } else if (0xDC00..=0xDFFF).contains(&unit) {
                (char::REPLACEMENT_CHARACTER, 1usize)
            } else {
                (
                    char::from_u32(u32::from(unit)).unwrap_or(char::REPLACEMENT_CHARACTER),
                    1usize,
                )
            };

            let bbox = if consumed == 1 {
                units[i].1
            } else {
                merge_two_bboxes(units[i].1, units[i + 1].1)
            };
            let rotation_degrees = units[i].2;

            objects.chars.push(Char {
                unicode_char: Some(scalar.to_string()),
                bbox,
                rotation_degrees: OrderedFloat::from(rotation_degrees),
                upright: rotation_degrees == 0.0 || rotation_degrees == 180.0,
            });

            i += consumed;
        }
        // if page_rotation_degrees == PdfPageRenderRotation::None {
        //     // for some pdf pages, their rotation is 0 degrees, but the characters are rotated
        //     self.deal_with_page_not_rotated_but_most_chars_rotated(objects);
        // }
    }

    fn bbox_for_text_char(&self, character: &PdfPageTextChar) -> BboxKey {
        let char_rect = character.loose_bounds().unwrap();
        let (x1, y1) = (char_rect.left(), char_rect.top());
        let (x2, y2) = (char_rect.right(), char_rect.bottom());

        let (y1, y2) = (
            self.get_v_coord_with_bottom_origin(y1.value),
            self.get_v_coord_with_bottom_origin(y2.value),
        );
        (
            OrderedFloat::from(x1.value),
            cmp::min(y1, y2),
            OrderedFloat::from(x2.value),
            cmp::max(y1, y2),
        )
    }

    /// Processes a path object and extracts lines or rectangles.
    ///
    /// # Arguments
    ///
    /// * `objects` - The Objects struct to populate.
    /// * `obj` - The PDF path object to process.
    /// * `parent_matrix` - Optional parent transformation matrix (e.g., from Form XObject).
    fn process_path_obj(
        &self,
        objects: &mut Objects,
        obj: &PdfPagePathObject,
        parent_matrix: Option<PdfMatrix>,
    ) {
        let n_segs = obj.segments().len();
        let mut points_vec = Vec::new();
        let mut points = Vec::with_capacity(n_segs as usize);

        // Apply transformation matrices
        // For Form XObject: point_in_page = form_matrix * path_matrix * point_in_form
        // The correct order is: obj_matrix.multiply(parent_matrix)
        let transform_matrix = if let Some(parent) = parent_matrix {
            let obj_matrix = obj.matrix().unwrap();
            obj_matrix.multiply(parent)
        } else {
            obj.matrix().unwrap()
        };

        for seg in obj.segments().transform(transform_matrix).iter() {
            let x = OrderedFloat::from(seg.x().value);
            let y = self.get_v_coord_with_bottom_origin(seg.y().value);

            if seg.segment_type() == PdfPathSegmentType::MoveTo {
                if !points.is_empty() {
                    points_vec.push(points.clone());
                    points.clear();
                }
                points.push(((x, y), PdfPathSegmentType::MoveTo));
            } else {
                points.push(((x, y), seg.segment_type()));
            }
        }

        if !points.is_empty() {
            points_vec.push(points.clone());
        }
        points.clear();

        for points in points_vec {
            if is_rect(&points) {
                let bbox = {
                    let x_values: Vec<OrderedFloat<f32>> = points.iter().map(|p| p.0.0).collect();
                    let y_values: Vec<OrderedFloat<f32>> = points.iter().map(|p| p.0.1).collect();
                    (
                        *x_values.iter().min().unwrap(),
                        *y_values.iter().min().unwrap(),
                        *x_values.iter().max().unwrap(),
                        *y_values.iter().max().unwrap(),
                    )
                };
                objects.rects.push(Rect {
                    bbox,
                    fill_color: obj.fill_color().unwrap(),
                    stroke_color: obj.stroke_color().unwrap(),
                    stroke_width: obj.stroke_width().unwrap().value,
                    is_stroked: obj.is_stroked().unwrap(),
                    fill_mode: obj.fill_mode().unwrap_or(PdfPathFillMode::None),
                });
            } else if points.len() == 2 && points[1].1 == PdfPathSegmentType::LineTo {
                objects.lines.push(Line {
                    line_type: LineType::Straight,
                    points: points.iter().map(|p| p.0).collect(),
                    stroke_color: obj.stroke_color().unwrap(),
                    fill_color: obj.fill_color().unwrap(),
                    width: OrderedFloat(obj.stroke_width().unwrap().value * 2.0),
                    is_stroked: obj.is_stroked().unwrap(),
                    fill_mode: obj.fill_mode().unwrap_or(PdfPathFillMode::None),
                });
            } else {
                // Check if all points except points[0] have LineTo as their second element
                let all_line_to = points[1..]
                    .iter()
                    .all(|p| p.1 == PdfPathSegmentType::LineTo);
                let line_type = if points.len() >= 3 && all_line_to {
                    LineType::Polyline
                } else {
                    LineType::Curve
                };
                objects.lines.push(Line {
                    line_type,
                    points: points.iter().map(|p| p.0).collect(),
                    stroke_color: obj.stroke_color().unwrap(),
                    fill_color: obj.fill_color().unwrap(),
                    width: OrderedFloat(obj.stroke_width().unwrap().value * 2.0),
                    is_stroked: obj.is_stroked().unwrap(),
                    fill_mode: obj.fill_mode().unwrap_or(PdfPathFillMode::None),
                });
            }
        }
    }

    /// Recursively processes XObject form objects.
    ///
    /// XObject forms can contain nested path objects and other forms.
    ///
    /// # Arguments
    ///
    /// * `objects` - The Objects struct to populate.
    /// * `obj` - The XObject form object to process.
    /// * `parent_matrix` - The accumulated transformation matrix from parent forms.
    fn process_x_object_form_obj(
        &self,
        objects: &mut Objects,
        obj: &PdfPageXObjectFormObject,
        parent_matrix: Option<PdfMatrix>,
    ) {
        if !obj.is_empty() {
            // Get the current form's matrix and compose with parent matrix
            // The correct order for nested forms: current_form_matrix.multiply(parent_matrix)
            let current_matrix = if let Ok(form_matrix) = obj.matrix() {
                if let Some(parent) = parent_matrix {
                    Some(form_matrix.multiply(parent))
                } else {
                    Some(form_matrix)
                }
            } else {
                parent_matrix
            };

            for sub_obj in obj.iter() {
                if let Some(sub_obj) = sub_obj.as_path_object() {
                    self.process_path_obj(objects, sub_obj, current_matrix);
                } else if let Some(sub_obj) = sub_obj.as_x_object_form_object() {
                    self.process_x_object_form_obj(objects, sub_obj, current_matrix);
                }
            }
        }
    }
}
