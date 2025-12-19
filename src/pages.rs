use crate::objects::*;
use ordered_float::OrderedFloat;
use pdfium_render::prelude::PdfPage as PdfiumPage;
use pdfium_render::prelude::*;
use std::cell::RefCell;
use std::cmp;
pub struct Page {
    pub inner: PdfiumPage<'static>,
    pub page_idx: usize,
    pub objects: RefCell<Option<Objects>>,
    bottom_origin: bool,
}

impl Page {
    pub fn new(inner: PdfiumPage<'static>, page_idx: usize, bottom_origin: bool) -> Self {
        let page = Self {
            inner,
            page_idx,
            objects: RefCell::new(None),
            bottom_origin,
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
    fn get_v_coord_with_bottom_origin(&self, y:f32) -> OrderedFloat<f32> {
        if self.bottom_origin {
            OrderedFloat::from(y)
        } else {
             OrderedFloat::from(self.height() - y)
        }
    }

    fn process_chars(&self, objects: &mut Objects) {
        let text = self.inner.text().unwrap();
        for character in text.chars().iter() {
            let char_rect = character.loose_bounds().unwrap();
            let (y1, y2) = (
                self.get_v_coord_with_bottom_origin(char_rect.top().value),
                self.get_v_coord_with_bottom_origin(char_rect.bottom().value),
            );
            let bbox = (
                OrderedFloat::from(char_rect.left().value),
                cmp::min(y1, y2),
                OrderedFloat::from(char_rect.right().value),
                cmp::max(y1, y2),
            );
            objects.chars.push(Char {
                unicode_char: character.unicode_string(),
                bbox: bbox,
            })
        }
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
                width: obj.stroke_width().unwrap().value * 2.0,
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
