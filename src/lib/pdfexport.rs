use std::collections::HashMap;
use std::path::PathBuf;
use image::ImageFormat::Jpeg;
use pdfium_render::error::PdfiumError;
use pdfium_render::page::PdfPoints;
use pdfium_render::pdfium::Pdfium;
use pdfium_render::prelude::{PdfPageContentRegenerationStrategy, PdfPagePaperSize, PdfRenderConfig};

pub struct PdfExport {
    pdfium: Pdfium
}

pub struct PdfExportResult {
    pub original_pdf: String,
    pub pages: Vec<PathBuf>,
}

impl PdfExport {
    pub fn new() -> Result<PdfExport, PdfiumError> {
        let pdfium_instance = Pdfium::new(
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./")).or_else(|_| Pdfium::bind_to_system_library())?,
        );
        return Ok(
            PdfExport{
                pdfium: pdfium_instance
            }
        )
    }

    pub fn export(&self, pdf_path: &String, out_path: &std::path::PathBuf, page_prefix: String, password: Option<&str>) -> Result<PdfExportResult, PdfiumError> {
        // Open the PDF document...
        let document = self.pdfium.load_pdf_from_file(pdf_path, password)?;

        let mut renderer_configs: HashMap<String, PdfRenderConfig> = HashMap::new();
        let pages_count = document.pages().len() as usize;
        let mut pages_paths: Vec<PathBuf> = Vec::with_capacity(pages_count);

        for (index, page) in document.pages().iter().enumerate() {
            let config_key = format!("{width}x{height}", width = page.page_size().width().value, height = page.page_size().height().value);
            if renderer_configs.contains_key(&config_key) == false {
                let render_config = PdfRenderConfig::new()
                    .set_target_width(page.page_size().width().value as u16)
                    .set_maximum_height(page.page_size().height().value as u16);
                renderer_configs.insert(config_key.clone(), render_config);
            }

            let output_page_path = out_path.join(format!("{}_[{}].jpg", page_prefix, index));

            page.render_with_config(&renderer_configs[&config_key])?
                .as_image() // Renders this page to an Image::DynamicImage...
                .as_rgba8() // ... then converts it to an Image::Image ...
                .ok_or(PdfiumError::ImageError)?
                .save_with_format(
                    &output_page_path,
                    Jpeg
                ) // ... and saves it to a file.
                .map_err(|_| PdfiumError::ImageError)?;

            pages_paths.push(output_page_path);
        }

        Ok( PdfExportResult { original_pdf: pdf_path.clone(), pages: pages_paths } )
    }

    pub fn glue(&self, pdf_output_path: &std::path::PathBuf, pages: &Vec<PathBuf>) -> Result<(), PdfiumError> {
        let document = self.pdfium.create_new_pdf()?;
        let mut doc_pages = document.pages();
        let mut index = 0;
        for page_image_path in pages {
            let image = image::open(page_image_path).unwrap();
            let points_width = PdfPoints::new(image.width() as f32);
            let points_height = PdfPoints::new(image.height() as f32);

            let mut new_page = doc_pages
                .create_page_at_index(PdfPagePaperSize::Custom(points_width.clone(), points_height.clone()), index)?;
            index += 1;
            new_page.set_content_regeneration_strategy(PdfPageContentRegenerationStrategy::AutomaticOnEveryChange);

            let origin_x = PdfPoints::new(0.0);
            let origin_y = PdfPoints::new(0.0);
            let _ = new_page.objects_mut().create_image_object(origin_x, origin_y, image, Some(points_width), Some(points_height));
        }
        return document.save_to_file(pdf_output_path);
    }
}