use crate::utils::{common_struct::BaseResponse, image::get_image_format_from_path};
use actix_multipart::Multipart;
use actix_web::{post, web, HttpResponse, Result};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self},
    io::Write,
    path::Path,
};
use uuid::Uuid;

#[derive(Serialize)]
pub struct UploadResponse {
    pub code: u16,
    pub message: String,
    pub url: String,
}

#[post("/api/upload")]
pub async fn upload(mut payload: Multipart) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    while let Some(item) = payload.next().await {
        let mut field = item?;
        let content_disposition = field.content_disposition();
        let original_name = content_disposition.get_filename().unwrap().to_string();

        let unique_id = Uuid::new_v4();

        let filename = format!("{}_{}", unique_id, original_name);
        let filepath = format!("./templates/{}", filename);

        let mut file = web::block(move || std::fs::File::create(filepath.clone()))
            .await?
            .unwrap();
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            file = web::block(move || file.write_all(&data).map(|_| file))
                .await?
                .unwrap();
        }

        let url = format!("/templates/{}", filename);
        return Ok(HttpResponse::Ok().json(UploadResponse {
            code: 200,
            message: "File uploaded successfully".to_string(),
            url,
        }));
    }

    Ok(HttpResponse::InternalServerError().json(UploadResponse {
        code: 500,
        message: "File upload failed".to_string(),
        url: "".to_string(),
    }))
}

#[derive(Deserialize)]
pub struct ResolutionInfo {
    resolution: Option<String>,
}

#[post("/api/upload-image")]
pub async fn upload_image(
    web::Query(info): web::Query<ResolutionInfo>,
    mut payload: Multipart,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    while let Some(item) = payload.next().await {
        let mut field = item?;
        let content_disposition = field.content_disposition();
        let original_name = content_disposition.get_filename().unwrap().to_string();
        let path = Path::new(&original_name);
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default();

        let unique_id = Uuid::new_v4();

        let original_filename = format!("{}_{}_original.{}", unique_id, stem, extension);
        let original_filepath = format!("./images/{}", original_filename);

        let filename = format!("{}_{}", unique_id, original_name);
        let filepath = format!("./images/{}", filename);

        let mut file = web::block(move || std::fs::File::create(filepath.clone()))
            .await?
            .unwrap();
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            file = web::block(move || file.write_all(&data).map(|_| file))
                .await?
                .unwrap();
        }

        match fs::copy(format!("./images/{}", filename), &original_filepath) {
            Ok(_) => {
                println!("File copied successfully!");
            }
            Err(e) => {
                println!("Failed to copy file: {}", e);
            }
        }

        // Resize the image if resolution parameter is provided
        if let Some(resolution) = &info.resolution {
            let parts: Vec<&str> = resolution.split('x').collect();
            if parts.len() == 2 {
                if let (Ok(width), Ok(height)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>())
                {
                    let img_path = format!("./storage/{}", filename);
                    match image::open(img_path) {
                        Ok(img) => {
                            // Check if the original image dimensions are smaller than the target dimensions
                            if img.width() < width || img.height() < height {
                                let url = format!("/images/{}", filename);
                                return Ok(HttpResponse::Ok().json(UploadResponse {
                                    code: 200,
                                    message: "Original image resolution is lower than the given resolution. No resizing performed.".to_string(),
                                    url
                                }));
                            }

                            let resized =
                                img.resize(width, height, image::imageops::FilterType::Lanczos3);
                            // Determine the format based on the original image's format
                            let format = get_image_format_from_path(
                                format!("./images/{}", filename).as_str(),
                            )
                            .unwrap_or(image::ImageFormat::Png);
                            if let Err(e) =
                                resized.save_with_format(format!("./images/{}", filename), format)
                            {
                                eprintln!("Resized image saving error: {}", e);
                                match fs::remove_file(format!("./images/{}", filename)) {
                                    Ok(_) => println!("File deleted successfully!"),
                                    Err(e) => println!("Error deleting file: {}", e),
                                };
                                return Ok(HttpResponse::InternalServerError().json(
                                    BaseResponse {
                                        code: 500,
                                        message: String::from("Error resizing image!"),
                                    },
                                ));
                            }
                        }
                        Err(e) => {
                            eprintln!("Image opening error: {}", e);
                            match fs::remove_file(format!("./images/{}", filename)) {
                                Ok(_) => println!("File deleted successfully!"),
                                Err(e) => println!("Error deleting file: {}", e),
                            };
                            return Ok(HttpResponse::InternalServerError().json(BaseResponse {
                                code: 500,
                                message: String::from("Error resizing image!"),
                            }));
                        }
                    }
                }
            }
        }

        let url = format!("/storage/{}", filename);
        return Ok(HttpResponse::Ok().json(UploadResponse {
            code: 200,
            message: "Image uploaded successfully".to_string(),
            url,
        }));
    }

    Ok(HttpResponse::InternalServerError().json(UploadResponse {
        code: 500,
        message: "Image upload failed".to_string(),
        url: "".to_string(),
    }))
}
