use std::{sync::Arc, path::{PathBuf, Path}};

use clap::Parser;
use headless_chrome::{Browser, Tab, types::Bounds};
use image::ImageFormat;

#[derive(Parser)]
struct Cli {
    input_file: String,
    output_dir: PathBuf,
}

fn read_file(filename: &str) -> Vec<String> {
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

fn take_screenshot(url: &str) -> Vec<u8> {
    let browser = Browser::default().unwrap();
    let tab = browser.wait_for_initial_tab().unwrap();
    set_dimensions(&tab, Some(1200.0), None);

    tab.navigate_to(url).unwrap()
        .wait_until_navigated().unwrap();
   
    let height = get_doc_height(&tab);
    set_dimensions(&tab, Some(1200.0), Some(height));

    let jpeg_data = tab.capture_screenshot(
        headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Jpeg,
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

    let urls = read_file(&args.input_file);
    for url in urls {
        let jpeg_bytes = take_screenshot(&url);
        let filename = format!(
            "{}.jpg", 
            url.replace(":", "_").replace("/", "_").replace(".", "_")
        );
        export_jpeg(jpeg_bytes, &args.output_dir.as_path(), filename);
    }
}