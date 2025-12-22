use actix_web::post;
use std::io::Error;

use actix_multipart::form::{
    MultipartForm,
    tempfile::{TempFile, TempFileConfig},
};
use actix_web::{App, HttpServer, Responder};
use uuid::Uuid;
#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(rename = "file", limit = "5MB")]
    files: Vec<TempFile>,
}

const PORT: u16 = 3696;
const HOST: &str = "127.0.0.1";

#[post("/")]
async fn save_files(
    MultipartForm(form): MultipartForm<UploadForm>,
) -> Result<impl Responder, Error> {
    let mut final_url = String::new();
    for f in form.files {
        let path: String;
        if let Some(s) = f.file_name {
            let filename = Uuid::new_v4().to_string().split_off(24);
            let split: Vec<_> = s.split(".").collect();
            if split.len() > 1 {
                let extension = split[split.len() - 1];
                path = format!("./tmp/{}.{}", filename, extension);
                final_url.push_str(&filename);
                final_url.push_str(".");
                final_url.push_str(&extension);
            } else {
                path = format!("./tmp/{}", filename);
                final_url.push_str(&filename);
            }
            f.file.persist(&path).ok();
        };
        final_url.push('\n');
    }
    Ok(format!("http://{}/{}", HOST, final_url))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::fs::create_dir_all("./tmp")?;
    println!("Starting up file server on port {PORT}");

    HttpServer::new(move || {
        App::new()
            .app_data(TempFileConfig::default().directory("./tmp"))
            .service(save_files)
    })
    .bind((HOST, PORT))?
    .run()
    .await
}
