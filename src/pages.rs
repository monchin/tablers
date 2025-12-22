use crate::objects::*;
use ordered_float::OrderedFloat;
use pdfium_render::prelude::PdfPage as PdfiumPage;
use pdfium_render::prelude::*;
use std::cell::RefCell;
use std::cmp;
use std::collections::HashMap;
pub struct Page {
    pub inner: PdfiumPage<'static>,
    pub page_idx: usize,
    pub objects: RefCell<Option<Objects>>,
    pub most_chars_rotation_degrees: RefCell<f32>,
}

impl Page {
    pub fn new(inner: PdfiumPage<'static>, page_idx: usize) -> Self {
        let page = Self {
            inner,
            page_idx,
            objects: RefCell::new(None),
            most_chars_rotation_degrees: RefCell::new(0.0),
        };
        page.extract_objects();
        page
    }

    pub fn clear(&self) {
        self.objects.replace(None);
    }

    pub fn width(&self) -> f32 {
        self.inner.width().value
    }

    pub fn height(&self) -> f32 {
        self.inner.height().value
    }

    pub fn rotation_degrees(&self) -> PdfPageRenderRotation {
        self.inner.rotation().unwrap()
    }

    pub fn extract_objects(&self) {
        if self.objects.borrow().is_none() {
            let objects = self.extract_objects_from_page();
            self.objects.replace(Some(objects));
        }
    }

    fn extract_objects_from_page(&self) -> Objects {
        let mut objects = Objects {
            rects: vec![],
            lines: vec![],
            chars: vec![],
        };
        self.process_chars(&mut objects);
        for obj in self.inner.objects().iter() {
            if let Some(obj) = obj.as_path_object() {
                self.process_path_obj(&mut objects, obj);
            } else if let Some(obj) = obj.as_x_object_form_object() {
                self.process_x_object_form_obj(&mut objects, obj);
            }
        }

        objects
    }

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

    fn count_chars_rotation(&self, chars: &[Char]) -> HashMap<u16, usize> {
        let mut result = HashMap::new();
        for char in chars {
            let rotation: u16 = char.rotation_degrees.round() as u16;
            let count = result.entry(rotation).or_insert(0);
            *count += 1;
        }
        result
    }

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

    fn process_chars(&self, objects: &mut Objects) {
        let text = self.inner.text().unwrap();

        for character in text.chars().iter() {
            let char_rect = character.loose_bounds().unwrap();
            let (x1, y1) = (char_rect.left(), char_rect.top());
            let (x2, y2) = (char_rect.right(), char_rect.bottom());

            let (y1, y2) = (
                self.get_v_coord_with_bottom_origin(y1.value),
                self.get_v_coord_with_bottom_origin(y2.value),
            );
            let bbox = (
                OrderedFloat::from(x1.value),
                cmp::min(y1, y2),
                OrderedFloat::from(x2.value),
                cmp::max(y1, y2),
            );
            let rotation_degrees = character.get_rotation_clockwise_degrees();

            objects.chars.push(Char {
                unicode_char: character.unicode_string(),
                bbox: bbox,
                rotation_degrees: OrderedFloat::from(rotation_degrees),
                upright: rotation_degrees == 0.0 || rotation_degrees == 180.0,
            })
        }
        // if page_rotation_degrees == PdfPageRenderRotation::None {
        //     // for some pdf pages, their rotation is 0 degrees, but the characters are rotated
        //     self.deal_with_page_not_rotated_but_most_chars_rotated(objects);
        // }
    }

    fn process_path_obj(&self, objects: &mut Objects, obj: &PdfPagePathObject) {
        let n_segs = obj.segments().len();
        let mut points = Vec::with_capacity(n_segs as usize);
        let mut line_type = LineType::Curve;
        for seg in obj.segments().transform(obj.matrix().unwrap()).iter() {
            let x = OrderedFloat::from(seg.x().value);
            let y = self.get_v_coord_with_bottom_origin(seg.y().value);

            points.push((x, y));
            if seg.segment_type() == PdfPathSegmentType::LineTo && n_segs == 2 {
                line_type = LineType::Straight;
            }
        }

        if is_rect(&points) {
            let bbox = {
                let x_values: Vec<OrderedFloat<f32>> = points.iter().map(|p| p.0).collect();
                let y_values: Vec<OrderedFloat<f32>> = points.iter().map(|p| p.1).collect();
                (
                    *x_values.iter().min().unwrap(),
                    *y_values.iter().min().unwrap(),
                    *x_values.iter().max().unwrap(),
                    *y_values.iter().max().unwrap(),
                )
            };
            objects.rects.push(Rect {
                bbox: bbox,
                fill_color: obj.fill_color().unwrap(),
                stroke_color: obj.stroke_color().unwrap(),
                stroke_width: obj.stroke_width().unwrap().value,
            });
        } else if points[0] != points[points.len() - 1] {
            objects.lines.push(Line {
                points: points,
                line_type: line_type,
                color: obj.stroke_color().unwrap(),
                width: OrderedFloat(obj.stroke_width().unwrap().value * 2.0),
            });
        }
    }

    fn process_x_object_form_obj(&self, objects: &mut Objects, obj: &PdfPageXObjectFormObject) {
        if !obj.is_empty() {
            for sub_obj in obj.iter() {
                if let Some(sub_obj) = sub_obj.as_path_object() {
                    self.process_path_obj(objects, sub_obj);
                } else if let Some(sub_obj) = sub_obj.as_x_object_form_object() {
                    self.process_x_object_form_obj(objects, sub_obj);
                }
            }
        }
    }
}
