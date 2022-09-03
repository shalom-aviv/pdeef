use std::{env, fs};
use std::cmp::max;
use std::path::PathBuf;
use uuid::{Uuid};
use pdfium_render::prelude::*;
use crate::options::CompareOptions;
use crate::pdfexport::{PdfExport, PdfExportResult};
use dify::{diff};
use dify::cli::OutputImageBase;

pub struct PdfDiff {
    pub base_pdf: String,
    pub diff_pdf: String,
    pub output_pdf: String,
    pub compare_options: CompareOptions,
}


struct TmpPdfDiffDirs {
    tmp_root: std::path::PathBuf,
    base_pdf_dir: std::path::PathBuf,
    diff_pdf_dir: std::path::PathBuf,
    diff_images_dir: std::path::PathBuf
}

impl PdfDiff {
    pub fn run(&self) -> Result<(), PdfiumError> {
        let (pdf_exporter, base_pages, diff_pages, tmp_folders) = self.extract_pdf_pages()?;

        let result_pdf_pages = self.create_diff_images(base_pages, diff_pages, &tmp_folders.diff_images_dir, &self.compare_options);

        pdf_exporter.glue(&PathBuf::from(&self.output_pdf), &result_pdf_pages)?;

        let _ = fs::remove_dir_all(tmp_folders.tmp_root);

        return Ok(());
    }

    fn tmp_folders() -> TmpPdfDiffDirs {
        let tmp_dir_uuid = Uuid::new_v4().to_string();
        let tmp_dir = env::temp_dir().join(tmp_dir_uuid.to_string());
        let base_tmp_dir = tmp_dir.join("base");
        let diff_tmp_dir = tmp_dir.join("diff");
        let diff_images_tmp_dir = tmp_dir.join("diff_images");

        let _ = fs::create_dir_all(&base_tmp_dir);
        let _ = fs::create_dir_all(&diff_tmp_dir);
        let _ = fs::create_dir_all(&diff_images_tmp_dir);

        return TmpPdfDiffDirs {
            tmp_root: tmp_dir,
            base_pdf_dir: base_tmp_dir,
            diff_pdf_dir: diff_tmp_dir,
            diff_images_dir: diff_images_tmp_dir
        };
    }

    fn extract_pdf_pages(&self) -> Result<(PdfExport, PdfExportResult, PdfExportResult, TmpPdfDiffDirs), PdfiumError> {
        let pdf_exporter = PdfExport::new()?;
        let folders = Self::tmp_folders();
        let base_prefix = String::from("base");
        let diff_prefix = String::from("diff");

        let base_pdf_export = pdf_exporter.export(
            &self.base_pdf,
            &folders.base_pdf_dir,
            base_prefix,
            None
        )?;

        let diff_pdf_export = pdf_exporter.export(
            &self.diff_pdf,
            &folders.diff_pdf_dir,
            diff_prefix,
            None
        )?;

        Ok((pdf_exporter, base_pdf_export, diff_pdf_export, folders))
    }

    fn create_diff_images(&self, base_pages: PdfExportResult, diff_pages: PdfExportResult, output_pages_dir: &PathBuf, compare_options: &CompareOptions) -> Vec<PathBuf> {
        let base_pages_count = base_pages.pages.len();
        let diff_pages_count = diff_pages.pages.len();
        let pages_count = max(base_pages_count, diff_pages_count);

        let mut pages: Vec<PathBuf> = Vec::with_capacity(pages_count);

        for index in 0..pages_count {
            if index < base_pages_count && index < diff_pages_count {
                //compare images and save result
                let result_page_path = output_pages_dir.join(format!("diff_result_page_[{}].png", index));
                self.diff(&base_pages.pages[index], &diff_pages.pages[index], &result_page_path, &compare_options);
                pages.push(result_page_path);
            } else if index >= base_pages_count {
                //use image from base
                pages.push(diff_pages.pages[index].clone());
            } else if index >= diff_pages_count {
                //use image from diff
                pages.push(base_pages.pages[index].clone());
            }
        }
        return pages;
    }

    fn diff(&self, left: &PathBuf, right: &PathBuf, output: &PathBuf, compare_options: &CompareOptions) -> Option<i32> {
        let output_image_base = OutputImageBase::LeftImage;
        let do_not_check_dimensions = true;
        let threshold = compare_options.threshold;
        let detect_anti_aliased_pixels = false;
        let blend_factor_of_unchanged_pixels = compare_options.alpha;
        // let block_out_areas = cli.get_block_out_area();

        let diff_params = &diff::RunParams {
            left: left.to_str().unwrap(),
            right: right.to_str().unwrap(),
            output: output.to_str().unwrap(),
            threshold: threshold,
            output_image_base: Some(output_image_base),
            do_not_check_dimensions: do_not_check_dimensions,
            detect_anti_aliased_pixels: detect_anti_aliased_pixels,
            blend_factor_of_unchanged_pixels: Some(blend_factor_of_unchanged_pixels),
            block_out_areas: None,
        };

        return match diff::run(diff_params) {
            Ok(value) => value,
            Err(error) => {
                println!("error: {:?}", error);
                return None
            }
        };
    }
}