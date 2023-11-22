use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};

use actix_web::{get, post, web, Error, HttpResponse, Responder};
use serde::Deserialize;
use tokio_postgres::Client;
use uuid::Uuid;

use crate::utils::{
    common_struct::{BaseResponse, DataResponse},
    html_parser::process_template,
    setting::get_port,
};

#[derive(Deserialize)]
pub struct ProcessReportRequest {
    pub template_name: String,
    pub options: SitetopdfOptions,
}

#[derive(Deserialize)]
pub struct SitetopdfOptions {
    format: Option<String>,
    landscape: Option<bool>,
    scale: Option<String>,
    margin_top: Option<String>,
    margin_bottom: Option<String>,
    margin_right: Option<String>,
    margin_left: Option<String>,
    header_template: Option<String>,
    footer_template: Option<String>,
    display_header_footer: Option<bool>,
    prefer_css_page_size: Option<bool>,
    page_ranges: Option<String>,
    ignore_http_errors: Option<bool>,
    wait_until: Option<String>,
    timeout: Option<String>,
    url: Option<String>,
    content: Option<String>,
    content_type: Option<String>,
    image: Option<bool>,
}

#[post("/api/site-to-pdf")]
pub async fn site_to_pdf(body: web::Json<SitetopdfOptions>) -> impl Responder {
    let mut sitetopdf = Command::new("sitetopdf");

    if body.url.is_none() && body.content.is_none() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Url or content must be set!"),
        });
    }

    if let Some(url) = &body.url {
        sitetopdf.arg("--url").arg(url);
    } else {
        if let Some(content) = &body.content {
            sitetopdf.arg("--content").arg(content);
        }
        if let Some(content_type) = &body.content_type {
            sitetopdf.arg("--content-type").arg(content_type);
        } else {
            sitetopdf.arg("--content-type").arg("string");
        }
    }

    let unique_id = Uuid::new_v4();
    let mut pdf_file_path = format!("./reports/{unique_id}.pdf");

    let mut url = String::from("/reports/{unique_id}.pdf");
    if let Some(image) = body.image {
        if image {
            sitetopdf.arg("--image");
            pdf_file_path = format!("./images/{unique_id}.png");
            sitetopdf.arg("--image-output").arg(pdf_file_path).arg("-v");
            url = String::from("/images/{unique_id}.png");
        }
    } else {
        sitetopdf.arg("-o").arg(pdf_file_path).arg("-v");
    }

    if let Some(format) = &body.format {
        sitetopdf.arg("--format").arg(format);
    }
    if let Some(landscape) = body.landscape {
        if landscape {
            sitetopdf.arg("--landscape");
        }
    }
    if let Some(scale) = &body.scale {
        sitetopdf.arg("--scale").arg(scale);
    }
    if let Some(margin_top) = &body.margin_top {
        sitetopdf.arg("--margin-top").arg(margin_top);
    }
    if let Some(margin_bottom) = &body.margin_bottom {
        sitetopdf.arg("--margin-bottom").arg(margin_bottom);
    }
    if let Some(margin_right) = &body.margin_right {
        sitetopdf.arg("--margin-right").arg(margin_right);
    }
    if let Some(margin_left) = &body.margin_left {
        sitetopdf.arg("--margin-left").arg(margin_left);
    }
    if let Some(header_template) = &body.header_template {
        sitetopdf.arg("--header-template").arg(header_template);
    }
    if let Some(footer_template) = &body.footer_template {
        sitetopdf.arg("--footer-template").arg(footer_template);
    }
    if let Some(display_header_footer) = body.display_header_footer {
        if display_header_footer {
            sitetopdf.arg("--display-header-footer");
        }
    }
    if let Some(prefer_css_page_size) = body.prefer_css_page_size {
        if prefer_css_page_size {
            sitetopdf.arg("--prefer-css-page-size");
        }
    }
    if let Some(page_ranges) = &body.page_ranges {
        sitetopdf.arg("--page-ranges").arg(page_ranges);
    }
    if let Some(ignore_http_errors) = body.ignore_http_errors {
        if ignore_http_errors {
            sitetopdf.arg("--ignore-http-errors");
        }
    }
    match &body.wait_until {
        Some(wait_until) => sitetopdf.arg("--wait-until").arg(wait_until),
        None => sitetopdf.arg("--wait-until").arg("load"),
    };
    if let Some(timeout) = &body.timeout {
        sitetopdf.arg("--timeout").arg(timeout);
    }

    let command = sitetopdf.output();

    match command {
        Ok(output) => {
            println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
            if !output.status.success() {
                // Convert stderr bytes to a string and print it
                let stderr_string = String::from_utf8_lossy(&output.stderr);
                println!("Command Error: {}", stderr_string);
                return HttpResponse::InternalServerError().json(BaseResponse {
                    code: 500,
                    message: "Failed to run sitetopdf command".to_string(),
                });
            }
            return HttpResponse::Ok().json(DataResponse {
                code: 200,
                message: String::from(""),
                data: Some(url),
            });
        }
        Err(err) => {
            println!("{:?}", err);
            return HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: "Error executing sitetopdf command".to_string(),
            });
        }
    }
}

#[post("/api/process-report")]
pub async fn process_report(
    body: web::Json<ProcessReportRequest>,
    client: web::Data<Arc<Client>>,
) -> impl Responder {
    let file_path = format!("./templates/{}", &body.template_name);

    if !Path::new(&file_path).exists() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Template not found!"),
        });
    }

    let html = match fs::read_to_string(file_path) {
        Ok(contents) => match process_template(&contents, &client).await {
            Ok(processed_html) => processed_html,
            Err(err) => {
                println!("{:?}", err);
                "".to_string()
            }
        },
        Err(err) => {
            println!("{:?}", err);
            return HttpResponse::BadRequest().json(BaseResponse {
                code: 400,
                message: String::from("Template could not be parsed!"),
            });
        }
    };
    println!("{}", html);

    if !fs::metadata("./reports").is_ok() {
        if let Err(_) = fs::create_dir_all("./reports") {
            return HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: "Internal Server Error".to_string(),
            });
        }
    }

    let unique_id = Uuid::new_v4();
    let processed_html_file_path = format!("./temp/{unique_id}.html");
    match File::create(&processed_html_file_path) {
        Ok(mut file) => {
            match file.write_all(html.as_bytes()) {
                Ok(_) => {
                    let mut sitetopdf = Command::new("sitetopdf");

                    let url = format!("http://localhost:{}/temp/{unique_id}.html", get_port());
                    sitetopdf.arg("--url").arg(url);

                    let pdf_file_path = format!("./reports/{unique_id}.pdf");
                    sitetopdf
                        .arg("--output")
                        .arg(pdf_file_path)
                        .arg("--verbose");

                    if let Some(format) = &body.options.format {
                        sitetopdf.arg("--format").arg(format);
                    }
                    if let Some(landscape) = body.options.landscape {
                        if landscape {
                            sitetopdf.arg("--landscape");
                        }
                    }
                    if let Some(scale) = &body.options.scale {
                        sitetopdf.arg("--scale").arg(scale);
                    }
                    if let Some(margin_top) = &body.options.margin_top {
                        sitetopdf.arg("--margin-top").arg(margin_top);
                    }
                    if let Some(margin_bottom) = &body.options.margin_bottom {
                        sitetopdf.arg("--margin-bottom").arg(margin_bottom);
                    }
                    if let Some(margin_right) = &body.options.margin_right {
                        sitetopdf.arg("--margin-right").arg(margin_right);
                    }
                    if let Some(margin_left) = &body.options.margin_left {
                        sitetopdf.arg("--margin-left").arg(margin_left);
                    }
                    if let Some(header_template) = &body.options.header_template {
                        sitetopdf.arg("--header-template").arg(header_template);
                    }
                    if let Some(footer_template) = &body.options.footer_template {
                        sitetopdf.arg("--footer-template").arg(footer_template);
                    }
                    if let Some(display_header_footer) = body.options.display_header_footer {
                        if display_header_footer {
                            sitetopdf.arg("--display-header-footer");
                        }
                    }
                    if let Some(prefer_css_page_size) = body.options.prefer_css_page_size {
                        if prefer_css_page_size {
                            sitetopdf.arg("--prefer-css-page-size");
                        }
                    }
                    if let Some(page_ranges) = &body.options.page_ranges {
                        sitetopdf.arg("--page-ranges").arg(page_ranges);
                    }
                    if let Some(ignore_http_errors) = body.options.ignore_http_errors {
                        if ignore_http_errors {
                            sitetopdf.arg("--ignore-http-errors");
                        }
                    }
                    match &body.options.wait_until {
                        Some(wait_until) => sitetopdf.arg("--wait-until").arg(wait_until),
                        None => sitetopdf.arg("--wait-until").arg("load"),
                    };
                    if let Some(timeout) = &body.options.timeout {
                        sitetopdf.arg("--timeout").arg(timeout);
                    }

                    let command = sitetopdf.output();

                    match command {
                        Ok(output) => {
                            println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
                            if !output.status.success() {
                                // Convert stderr bytes to a string and print it
                                let stderr_string = String::from_utf8_lossy(&output.stderr);
                                println!("Command Error: {}", stderr_string);
                                return HttpResponse::InternalServerError().json(BaseResponse {
                                    code: 500,
                                    message: "Failed to run sitetopdf command".to_string(),
                                });
                            }
                            tokio::spawn(async move {
                                match fs::remove_file(&processed_html_file_path) {
                                    Ok(_) => println!("File deleted successfully!"),
                                    Err(e) => println!("Error deleting file: {}", e),
                                };
                            });
                            return HttpResponse::Ok().json(DataResponse {
                                code: 200,
                                message: String::from(""),
                                data: Some(format!("/reports/{unique_id}.pdf")),
                            });
                        }
                        Err(err) => {
                            println!("{:?}", err);
                            return HttpResponse::InternalServerError().json(BaseResponse {
                                code: 500,
                                message: "Error executing sitetopdf command".to_string(),
                            });
                        }
                    }
                }
                Err(err) => {
                    println!("{:?}", err);
                    return HttpResponse::InternalServerError().json(BaseResponse {
                        code: 500,
                        message: String::from("Error writing template!"),
                    });
                }
            };
        }
        Err(err) => {
            println!("{:?}", err);
            return HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error creating output file!"),
            });
        }
    };
}

#[get("/reports/{file_name}")]
pub async fn get_report(path: web::Path<String>) -> Result<HttpResponse, Error> {
    let file_name = path.into_inner();
    // Check if the file exists
    let file_path = format!("./reports/{file_name}");
    if !PathBuf::from(&file_path).exists() {
        return Ok(HttpResponse::NotFound().finish());
    }

    // Serve the PDF file
    Ok(HttpResponse::Ok().body(actix_web::web::Bytes::from(std::fs::read(file_path)?)))
}
