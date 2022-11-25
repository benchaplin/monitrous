use std::env;
use std::error::Error;
use std::fs;

use headless_chrome::Browser;
use headless_chrome::protocol::browser::Bounds;
use headless_chrome::protocol::page::ScreenshotFormat;

use image::ImageFormat;

fn read_file(filename: &str) -> Vec<String> {
    let contents = fs::read_to_string(filename).unwrap();
    return contents.lines().map(|x| String::from(x)).collect();
}

fn take_screenshot(url: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let browser = Browser::default()?;
    let tab = browser.wait_for_initial_tab()?;
    tab.navigate_to(url)?
        .set_bounds(Bounds::Normal{ 
            left: Some(0), 
            top: Some(0), 
            width: Some(1200), 
            height: Some(1200) 
        })?
        .wait_until_navigated()?;

    let jpeg_data = tab.capture_screenshot(
        ScreenshotFormat::JPEG(None),
        None,
        true 
    )?;

    Ok(jpeg_data)
}

fn export_jpeg(jpeg_bytes: Vec<u8>, filename: String) {
    match image::load_from_memory_with_format(&jpeg_bytes, ImageFormat::Jpeg) {
        Ok(_) => {
            fs::write(filename, jpeg_bytes).unwrap();
        }
        Err(error) => {
            panic!("error exporting jpg: {:?}", error);
        }
    } 
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let urls = read_file(&args[1]);
    for url in urls {
        match take_screenshot(&url) {
            Ok(jpeg_bytes) => {
                let filename = format!(
                    "{}.jpg", 
                    url.replace(":", "_").replace("/", "_").replace(".", "_")
                );
                export_jpeg(jpeg_bytes, filename);
            }
            Err(_) => {
                println!("failed to screenshot {}", url);
            }
        }
    }
}
