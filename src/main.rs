use std::path::{Path, PathBuf};
use std::sync::Arc;

use clap::{Parser, Subcommand};
use dssim::{Dssim, Val};
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use headless_chrome::types::Bounds;
use headless_chrome::{Browser, Tab};
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
    action: Action,
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
        old_dir: PathBuf,
    },
}

fn read_file(filename: &Path) -> Vec<String> {
    let contents = std::fs::read_to_string(filename).unwrap();
    return contents.lines().map(|x| String::from(x)).collect();
}

fn set_dimensions(tab: &Arc<Tab>, width: Option<f64>, height: Option<f64>) {
    tab.set_bounds(Bounds::Normal {
        left: Some(0),
        top: Some(0),
        width,
        height,
    })
    .ok();
}

fn get_doc_height(tab: &Arc<Tab>) -> f64 {
    let doc = tab.wait_for_element("html").unwrap();
    let remote_height = doc
        .call_js_fn(
            r#"
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
    "#,
            Vec::new(),
            false,
        )
        .unwrap();

    remote_height.value.unwrap().as_f64().unwrap()
}

fn take_screenshot(tab: &Arc<Tab>, url: &str) -> Vec<u8> {
    set_dimensions(&tab, Some(1200.0), None);

    tab.navigate_to(url)
        .unwrap()
        .wait_until_navigated()
        .unwrap();

    let height = get_doc_height(&tab);
    set_dimensions(&tab, Some(1200.0), Some(height));

    let png_data = tab
        .capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, true)
        .unwrap();

    png_data
}

fn export_png(png_bytes: Vec<u8>, output_dir: &Path, filename: String) {
    image::load_from_memory_with_format(&png_bytes, ImageFormat::PNG).unwrap();
    std::fs::create_dir_all(output_dir).ok();
    std::fs::write(
        format!("{}/{}", output_dir.to_str().unwrap(), filename),
        png_bytes,
    )
    .unwrap();
}

fn take_screenshots(urls: Vec<String>, output_dir: &Path) {
    let browser = Browser::default().unwrap();

    let pb = ProgressBar::new(urls.len() as u64);
    for url in urls {
        let tab = browser.new_tab().unwrap();
        let png_bytes = take_screenshot(&tab, &url);
        let filename = format!(
            "{}.png",
            url.replace(":", "_").replace("/", "_").replace(".", "_")
        );
        export_png(png_bytes, output_dir, filename);
        pb.println(format!("captured {}", url));
        pb.inc(1);
    }
}

fn compare_imgs(img1: &Path, img2: &Path) -> Val {
    let img1_size = imagesize::size(img1).unwrap();
    let img2_size = imagesize::size(img2).unwrap();
    if img1_size.width != img2_size.width || img1_size.height != img2_size.height {
        return Val::new(1.0);
    }

    let attr = Dssim::new();
    let dssim_img1 = dssim::load_image(&attr, img1).unwrap();
    let dssim_img2 = dssim::load_image(&attr, img2).unwrap();
    let (diff, _) = attr.compare(&dssim_img1, &dssim_img2);

    diff
}

fn save_diff_img(img1_path: &Path, img2_path: &Path, output_path: &Path) {
    let mut img1 = image::open(img1_path).unwrap();
    let mut img2 = image::open(img2_path).unwrap();

    let img_diff = lcs_image_diff::compare(&mut img1, &mut img2, 100.0 / 256.0).unwrap();

    img_diff.save(output_path).unwrap();
}

fn compare_screenshots(old_dir: &Path, new_dir: &Path) {
    let old_paths = std::fs::read_dir(old_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path().file_name().unwrap().to_str().unwrap().to_string())
        .collect::<Vec<_>>();
    let new_paths = std::fs::read_dir(new_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path().file_name().unwrap().to_str().unwrap().to_string())
        .collect::<Vec<_>>();

    let pb = ProgressBar::new(new_paths.len() as u64);
    for path in new_paths {
        if old_paths.contains(&path) {
            let old_path = &old_dir.join(&path);
            let new_path = &new_dir.join(&path);

            pb.println(format!("comparing {}", &path));
            let diff = compare_imgs(old_path, new_path);
            if diff > 0.0 {
                pb.println(format!(
                    "diff: {}, generating diff image for {}",
                    diff, &path
                ));
                save_diff_img(old_path, new_path, Path::new(&path));
            }
            pb.inc(1);
        }
    }
}

fn main() {
    let args = Cli::parse();

    match &args.action {
        Action::Capture {
            input_file,
            output_dir,
        } => {
            let urls = read_file(&input_file.as_path());
            take_screenshots(urls, &output_dir.as_path());
        }
        Action::Compare { new_dir, old_dir } => {
            compare_screenshots(old_dir, new_dir);
        }
    }
}
