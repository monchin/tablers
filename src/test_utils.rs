#[cfg(test)]
use pdfium_render::prelude::Pdfium;
#[cfg(test)]
use std::sync::OnceLock;

#[cfg(test)]
static PDFIUM: OnceLock<Pdfium> = OnceLock::new();

#[cfg(test)]
pub fn load_pdfium() -> &'static Pdfium {
    PDFIUM.get_or_init(|| {
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
    })
}
