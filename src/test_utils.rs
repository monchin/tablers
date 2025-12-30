#[cfg(test)]
use pdfium_render::prelude::Pdfium;

/// Gets a reference to the global Pdfium instance.
/// This delegates to the global PDFIUM in lib.rs to ensure only one instance exists.
#[cfg(test)]
pub fn load_pdfium() -> &'static Pdfium {
    crate::get_or_init_pdfium()
}
