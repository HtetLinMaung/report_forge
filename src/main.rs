extern crate dotenv;

use actix_cors::Cors;
use actix_files as fs;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use utils::setting;

mod api;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let port: u16 = setting::get_port();
    let client = setting::get_postgres_connection().await;

    HttpServer::new(move || {
        // let default_size = env::var("DEFAULT_REQUEST_SIZE")
        //     .unwrap_or_else(|_| "2097152".to_string())
        //     .parse::<usize>()
        //     .unwrap_or(2097152);
        let cors = Cors::permissive() // This allows all origins. Be careful with this in production!
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        if !std::fs::metadata("./temp").is_ok() {
            if let Err(err) = std::fs::create_dir_all("./temp") {
                println!("{:?}", err);
            }
        }
        if !std::fs::metadata("./templates").is_ok() {
            if let Err(err) = std::fs::create_dir_all("./templates") {
                println!("{:?}", err);
            }
        }
        App::new()
            .wrap(cors)
            .app_data(web::Data::new(client.clone()))
            .configure(api::init)
            .service(fs::Files::new("/temp", "./temp").show_files_listing())
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
