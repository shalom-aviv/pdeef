use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CliArgs {
    /// Path to pdf to compare from
    pub base_path: String,

    /// Path to pdf to compare to
    pub diff_path: String,

    /// Path to output pdf
    #[structopt(long = "output", short = "o", default_value = "./pdeef-output.pdf")]
    pub output_path: String,

    /// The color of differing pixels in [R, G, B, A] format
    #[structopt(long = "diffColour", short = "c", default_value = "[218, 165, 32, 255]", parse(try_from_str="parse_num_array"))]
    pub diff_colour: [u32; 4],

    /// Matching threshold, smaller values makes pixel comparison more sensitive
    #[structopt(long = "threshold", short = "t", default_value = "0.1")]
    pub threshold: f32,

    /// Blending value of unchaged pixels
    #[structopt(long = "alpha", short = "a", default_value = "0.6")]
    pub alpha: f32
}

fn parse_num_array(array: &str) -> Result<[u32; 4], &'static str> {
    let array = array.trim_start_matches("[")
        .trim_end_matches("]")
        .split(",");

    let mut num_array: [u32; 4] = [255; 4];
    for (i, el) in array.enumerate() {
        num_array[i] = el.trim().parse::<u32>().expect("Argument incorrectly formatted, correct format should be: [a, b, c]");
    }

    Ok(num_array)
}
