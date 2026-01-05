use std::{fs, time::Duration};

use actix_files::Files;
use actix_multipart::form::{
    MultipartForm,
    tempfile::{TempFile, TempFileConfig},
    text::Text,
};
use actix_web::{
    App, Error, HttpResponse, HttpServer, Responder, post,
    web::{self},
};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use hightower_naming::generate_random_name;

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(limit = "5MB")]
    file: TempFile,
    time: Option<Text<u64>>,
    oneshot: Option<Text<bool>>,
}

const PORT: &'static str = env!("lbin_port");
const HOST: &'static str = env!("lbin_host");
const AUTH: &'static str = env!("lbin_auth");
const URL: &'static str = env!("lbin_url");

#[post("/")]
async fn default_post(
    MultipartForm(form): MultipartForm<UploadForm>,
    cred: BearerAuth,
) -> Result<impl Responder, Error> {
    if cred.token() != AUTH {
        Ok(HttpResponse::Unauthorized().body("Invalid auth token.\n"))
    } else {
        let (path, file) = file_helper(&form.file);
        // let url = format!("{}/{}", URL, &file);
        if let Some(b) = form.oneshot {
            if b.0 {
                let url = format!("http://localhost:3696/o/{}", &file);
                form.file.file.persist(&path).ok();
                Ok(HttpResponse::Ok().body(url))
            } else {
                Ok(HttpResponse::Ok().body("Oneshot is not needed."))
            }
        } else {
            let url = format!("http://localhost:3696/{}", &file);
            let expiry = if let Some(n) = form.time { n.0 } else { 6 * 60 };
            let mut interval = tokio::time::interval(Duration::from_mins(expiry));
            form.file.file.persist(&path).ok();
            tokio::spawn(async move {
                interval.tick().await;
                interval.tick().await;
                tokio::fs::remove_file(path)
            });
            Ok(HttpResponse::Ok().body(url))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    fs::create_dir_all("./tmp")?;
    println!("Starting up file server on port {HOST}:{PORT}");
    HttpServer::new(|| {
        App::new()
            .app_data(TempFileConfig::default().directory("./tmp"))
            .service(default_post)
            .service(Files::new("/", "./tmp").index_file("../index.html"))
            .default_service(web::to(|| async {
                HttpResponse::NotFound().body("File expired or does not exist.")
            }))
    })
    .bind((HOST, PORT.parse().expect("Error converting port to number")))?
    .run()
    .await
}

fn file_helper(file: &TempFile) -> (String, String) {
    let mut path = String::new();
    let mut final_file = String::new();
    if let Some(f) = &file.file_name {
        let filename = generate_random_name(None);
        let split: Vec<_> = f.split(".").collect();
        if split.len() > 1 {
            let extension = split[split.len() - 1];
            path = format!("./tmp/{}.{}", &filename, &extension);
            final_file = format!("{}.{}\n", &filename, &extension);
        } else {
            path = format!("./tmp/{}", &filename);
            final_file = format!("{}\n", &filename);
        }
    }
    (path, final_file)
}
