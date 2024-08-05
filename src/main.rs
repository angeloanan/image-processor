use std::ffi::OsStr;

use image::{GenericImageView, ImageReader};
use tracing::{error, info, instrument};

const SUPPORTED_EXTENSIONS: [&str; 4] = ["jpg", "jpeg", "png", "dng"];

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    check_exiftool().await;

    let dir = std::fs::read_dir("pics").expect("Unable to read the `pics` directory");
    for entry in dir {
        if entry.is_err() {
            continue;
        }

        let entry = entry.unwrap();
        let path = entry.path();
        let ext = path.extension();
        if path.is_dir()
            || ext.is_none()
            || !SUPPORTED_EXTENSIONS.contains(&ext.unwrap().to_str().unwrap())
        {
            continue;
        };

        if ext == Some(OsStr::new("dng")) {
            continue;
        }

        process_image(path).await;
    }
}

#[instrument]
async fn check_exiftool() {
    let exiftool = tokio::process::Command::new("exiftool")
        .arg("-ver")
        .output()
        .await
        .expect("Unable to run exiftool");

    if exiftool.status.success() {
        let version = String::from_utf8(exiftool.stdout).unwrap();
        let version = version.trim();
        info!("Using exiftool {version}",);
    } else {
        panic!("exiftool is not installed or not in the PATH");
    }
}

#[instrument]
async fn process_image(path: std::path::PathBuf) {
    info!("Processing {}", path.display());

    let image = ImageReader::open(&path).expect("Unable to open the image");
    let decoded_image = image.decode().expect("Unable to decode the image");

    info!("Image dimensions: {:?}", decoded_image.dimensions());

    let exif = tokio::process::Command::new("exiftool")
        .arg(&path)
        .output()
        .await;
    if exif.is_err() {
        error!("Something went wrong when calling exiftool");
        error!("{}", exif.err().unwrap());
        return;
    }

    let exif = exif.unwrap();
    if exif.status.success() {
        let data = String::from_utf8(exif.stdout).unwrap();
        info!("EXIF data: {data}");
    } else {
        info!("No EXIF data found");
        return;
    }
}
