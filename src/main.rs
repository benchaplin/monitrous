use std::path::{PathBuf, Path};
use std::sync::Arc;

use clap::{Parser, Subcommand};
use headless_chrome::{Browser, Tab};
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use headless_chrome::types::Bounds;
use image::ImageFormat;
use indicatif::ProgressBar;

/// Monitor a list of URLs by capturing screenshots and comparing them 
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[clap(override_usage = "
  monitrous capture <INPUT_FILE> <OUTPUT_DIR>
  monitrous compare <NEW_DIR> <OLD_DIR>")]
struct Cli {
    #[command(subcommand)]
    action: Action
}

#[derive(Subcommand, Debug)]
enum Action {
    /// Captures screenshots of given URLs
    Capture {
        /// Input file of URLs for capture (separated by line)
        #[clap(value_parser)]
        input_file: PathBuf,
        /// Output directory for captured screenshots
        #[clap(value_parser)]
        output_dir: PathBuf,
    },
    /// Compares screenshots in given two directories
    Compare {
        /// New directory for comparison
        #[clap(value_parser)]
        new_dir: PathBuf,
        /// Old directory for comparison
        #[clap(value_parser)]
        old_dir: PathBuf
    }
}


fn read_file(filename: &Path) -> Vec<String> {
    let contents = std::fs::read_to_string(filename).unwrap();
    return contents.lines().map(|x| String::from(x)).collect();
}

fn set_dimensions(tab: &Arc<Tab>, width: Option<f64>, height: Option<f64>) {
    tab.set_bounds(Bounds::Normal{ 
        left: Some(0), 
        top: Some(0), 
        width,
        height
    }).ok(); 
}

fn get_doc_height(tab: &Arc<Tab>) -> f64 {
    let doc = tab.wait_for_element("html").unwrap();
    let remote_height = doc.call_js_fn(r#"
        function getPageHeight() {
            const body = document.body;
            const html = document.documentElement;
            return Math.max(
                body.scrollHeight, 
                body.offsetHeight, 
                html.clientHeight, 
                html.scrollHeight, 
                html.offsetHeight 
            );
        }
    "#, Vec::new(), false).unwrap();

    remote_height.value.unwrap().as_f64().unwrap()
}

fn take_screenshots(urls: Vec<String>, output_dir: &Path) {
    let browser = Browser::default().unwrap();

    let pb = ProgressBar::new(urls.len() as u64);
    for url in urls {
        let tab = browser.new_tab().unwrap();
        let jpeg_bytes = take_screenshot(&tab, &url);
        let filename = format!(
            "{}.jpg", 
            url.replace(":", "_").replace("/", "_").replace(".", "_")
        );
        export_jpeg(jpeg_bytes, output_dir, filename);
        pb.println(format!("captured {}", url));
        pb.inc(1);
    }
}

fn take_screenshot(tab: &Arc<Tab>, url: &str) -> Vec<u8> {
    set_dimensions(&tab, Some(1200.0), None);

    tab.navigate_to(url).unwrap()
        .wait_until_navigated().unwrap();
   
    let height = get_doc_height(&tab);
    set_dimensions(&tab, Some(1200.0), Some(height));

    let jpeg_data = tab.capture_screenshot(
        CaptureScreenshotFormatOption::Jpeg,
        None,
        None,
        true 
    ).unwrap();

    jpeg_data
}

fn export_jpeg(jpeg_bytes: Vec<u8>, output_dir: &Path, filename: String) {
    image::load_from_memory_with_format(&jpeg_bytes, ImageFormat::Jpeg).unwrap();
    std::fs::create_dir_all(output_dir).ok();
    std::fs::write(
        format!("{}/{}", output_dir.to_str().unwrap(), filename), 
        jpeg_bytes
    ).unwrap();
}

fn main() {
    let args = Cli::parse();

    match &args.action {
        Action::Capture { input_file, output_dir } => {
            let urls = read_file(&input_file.as_path());
            take_screenshots(urls, &output_dir.as_path());
        }
        Action::Compare { new_dir, old_dir } => {
            println!("comparing {} to {}", new_dir.as_path().display(), old_dir.as_path().display());
        }
    }
}