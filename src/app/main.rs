mod cliargs;

use structopt::StructOpt;
use cliargs::CliArgs;
use pdeef::{options::CompareOptions};
use pdeef::{pdfdiff::PdfDiff};

fn main() {
    let opt = CliArgs::from_args();
    println!("opt: {:?}", opt);

    let base = String::from(opt.base_path);
    let diff = String::from(opt.diff_path);
    let output = String::from(opt.output_path);
    // let base_pdf = read_image_from_file(&base);
    // let diff_pdf = read_image_from_file(&diff);

    let options = CompareOptions {
        threshold: opt.threshold,
        alpha: opt.alpha,
        diff_colour: opt.diff_colour
    };

    let pdf_diff = PdfDiff  {
        base_pdf: base,
        diff_pdf: diff,
        output_pdf: output,
        compare_options: options,
    };

    match pdf_diff.run() {
        Ok(_) => (),
        Err(error) => {
            println!("error: {:?}", error);
            panic!("PdfiumError");
        }
    }
}