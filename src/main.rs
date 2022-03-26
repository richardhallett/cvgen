use std::env;
use std::fs;
use pulldown_cmark::{Parser as MParser, Options, html};
use clap::Parser;

// Constant for data directory
const DATA_DIR: &str = "data";
// Constant for output directory
const OUTPUT_DIR: &str = "out";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// The path to the markdown file to read
    #[clap(parse(from_os_str))]
    input_path: std::path::PathBuf,

    /// The output file path to write to
    /// Defaults to the input file with .html extension.
    #[clap(short, long)]
    output_path: Option<std::path::PathBuf>,

    /// Path to base html file used to generate the output
    /// Defaults to the base.html file in the data directory
    #[clap(short, long)]
    base_html_file: Option<std::path::PathBuf>,

    // Switch to output pdf using wkhtmltopdf
    #[clap(long)]
    pdf: bool,

    /// The street address to be used to replace on the input file $address
    #[clap(short, long)]
    address: String,

    /// The contact phone number to be used to replace on the input file $phone_number
    #[clap(short, long)]
    phone: String,
}


fn main() {
    let args = Cli::parse();
    let input_path = args.input_path;
    let address = args.address;
    let phone = args.phone;

    // If the output path is not specified, use the input path with .html extension
    let output_path = args.output_path.unwrap_or_else(|| {
        let mut path = input_path.clone();
        path.set_extension("html");
        path
    });

    // Read the markdown file from the input path
    let markdown_content = fs::read_to_string(input_path)
        .expect("Can't read markdown file");

    // Replace the placeholders with the CLI arguments
    let markdown_content = markdown_content.replace("$address", &address);
    let markdown_content = markdown_content.replace("$phone_number", &phone);

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);

    let parser = MParser::new_ext(&markdown_content, options);

    // Write to String buffer.
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    // Get the base html file path from the CLI or the default
    let base_html_file_path = args.base_html_file.unwrap_or_else(|| {
        let mut path = env::current_dir().unwrap();
        path.push(DATA_DIR);
        path.push("base.html");
        path
    });

    // Read base html file from the data directory
    let base_html = fs::read_to_string(base_html_file_path)
        .expect("Can't read base html file");

    // Replace the placeholders with the html output
    let html_output = base_html.replace("$content", &html_output);

    fs::create_dir_all(OUTPUT_DIR).expect("Can't create output directory");
    fs::write(&output_path, html_output).expect("Can't write to output file");

    // If the pdf flag is set, use wkhtmltopdf to convert the html to pdf
    if args.pdf {
        let mut pdf_path = output_path.clone();
        pdf_path.set_extension("pdf");
        let command = format!("wkhtmltopdf {} {}", output_path.to_str().unwrap(), pdf_path.to_str().unwrap());
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .expect("Can't execute wkhtmltopdf");
        println!("{}", String::from_utf8_lossy(&output.stdout));
    }
}