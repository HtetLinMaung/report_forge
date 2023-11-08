use actix_web::web;

mod file;
mod report;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(file::upload);
    cfg.service(file::upload_image);
    cfg.service(report::site_to_pdf);
    cfg.service(report::process_report);
    cfg.service(report::get_report);
}
