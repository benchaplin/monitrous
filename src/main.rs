use std::path::{Path, PathBuf};
use std::sync::Arc;

// use anyhow::{Context, Result};
use clap::Parser;
use headless_chrome::{Browser, Tab};
use headless_chrome::protocol::browser::Bounds;
use headless_chrome::protocol::page::ScreenshotFormat;
use image::ImageFormat;
use indicatif::ProgressBar;

#[derive(Parser)]
struct Cli {
    input_file: String,
    output_dir: PathBuf,
}

fn read_file(filename: &str) -> Vec<String> {
    let contents = std::fs::read_to_string(filename).unwrap();
    return contents.lines().map(|x| String::from(x)).collect();
}

fn set_dimensions(tab: &Arc<Tab>, width: Option<u32>, height: Option<u32>) {
    tab.set_bounds(Bounds::Normal{ 
        left: Some(0), 
        top: Some(0), 
        width,
        height
    }).ok();
}

fn get_doc_height(tab: &Arc<Tab>) -> u32 {
    let doc = tab.wait_for_element("div#app").unwrap();
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
    "#, false).unwrap();

    remote_height.value.unwrap().as_u64().unwrap() as u32
}

fn take_screenshot(tab: &Arc<Tab>) -> Vec<u8> {
    let height = get_doc_height(&tab);
    println!("height {}", height); 
    set_dimensions(&tab, Some(1200), Some(height));

    let jpeg_data = tab.capture_screenshot(
        ScreenshotFormat::JPEG(None),
        None,
        true 
    ).unwrap();

    jpeg_data
}

fn take_screenshots(urls: Vec<String>, output_dir: &Path) {
    let browser = Browser::default().unwrap();
    let tab = browser.wait_for_initial_tab().unwrap();
    set_dimensions(&tab, Some(1200), None);

    let pb = ProgressBar::new(urls.len() as u64);
    for url in urls {
        tab.navigate_to(&url).unwrap()
            .wait_until_navigated().unwrap();

        let jpeg_bytes = take_screenshot(&tab);

        let filename = format!(
            "{}.jpg", 
            url.replace(":", "_").replace("/", "_").replace(".", "_")
        );
        export_jpeg(jpeg_bytes, output_dir, &filename);

        pb.println(format!("captured {}", url));
        pb.inc(1);
    }
    pb.finish_with_message("done");
}

fn export_jpeg(jpeg_bytes: Vec<u8>, output_dir: &Path, filename: &str) {
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
    take_screenshots(urls, &args.output_dir.as_path());
}
