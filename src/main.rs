use std::{env, fs, time::Duration};

use actix_files::Files;
use actix_multipart::form::{
    MultipartForm,
    tempfile::{TempFile, TempFileConfig},
    text::Text,
};
use actix_web::{
    App, Error, HttpResponse, HttpServer, Responder, get, post,
    web::{self},
};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use hightower_naming::generate_random_name;

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(limit = "5MB")]
    file: TempFile,
    time: Option<Text<u64>>,
}

const PORT: &'static str = env!("port");
const HOST: &'static str = env!("host");
const AUTH: &'static str = env!("auth");
const URL: &'static str = env!("url");
const TMP: &'static str = env!("tmp");
const OS: &'static str = env!("os");

#[get("/o/{filename}")]
async fn oneshot_get(path: web::Path<String>) -> Result<impl Responder, Error> {
    let filename = path.into_inner();
    let path = env::current_dir()?.display().to_string();
    let full_path = format!("{}/{OS}/{filename}", path);
    let fp = full_path.to_owned();
    let content = web::block(move || fs::read(full_path)).await??;
    web::block(move || std::fs::remove_file(fp)).await??;
    Ok(HttpResponse::Ok().body(content))
}

#[post("/o")]
async fn oneshot_post(
    MultipartForm(form): MultipartForm<UploadForm>,
    cred: BearerAuth,
) -> Result<impl Responder, Error> {
    if cred.token() != AUTH {
        Ok(HttpResponse::Unauthorized().body("Invalid auth token.\n"))
    } else {
        let file = file_helper(&form.file);
        let path = std::env::current_dir()?.display().to_string();
        let full_path = format!("{path}/{OS}/{file}");
        let url = format!("{}/o/{}\n", URL, &file);
        // let dev_url = format!("http://{HOST}:{PORT}/o/{}\n", file);
        form.file.file.persist(full_path).ok();
        Ok(HttpResponse::Ok().body(url))
    }
}

#[post("/")]
async fn default_post(
    MultipartForm(form): MultipartForm<UploadForm>,
    cred: BearerAuth,
) -> Result<impl Responder, Error> {
    if cred.token() != AUTH {
        Ok(HttpResponse::Unauthorized().body("Invalid auth token.\n"))
    } else {
        let path = std::env::current_dir()?.display().to_string();
        let file = file_helper(&form.file);
        let full_path = format!("{path}/{TMP}/{file}");
        let url = format!("{}/{file}\n", URL);
        // let dev_url = format!("http://{HOST}:{PORT}/{}\n", &file);
        let expiry = if let Some(n) = form.time { n.0 } else { 6 * 60 };
        let mut interval = tokio::time::interval(Duration::from_mins(expiry));
        form.file.file.persist(&full_path).ok();
        tokio::spawn(async move {
            interval.tick().await;
            interval.tick().await;
            std::fs::remove_file(full_path)
        });
        Ok(HttpResponse::Ok().body(url))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    fs::create_dir_all(TMP)?;
    fs::create_dir_all(OS)?;
    println!("Starting up file server on port {HOST}:{PORT}");
    HttpServer::new(|| {
        App::new()
            .app_data(TempFileConfig::default().directory(TMP))
            .service(default_post)
            .service(oneshot_post)
            .service(oneshot_get)
            .service(Files::new("/", TMP).index_file("../index.html"))
            .service(Files::new("/o", OS).index_file("../index.html"))
            .default_service(web::to(|| async {
                HttpResponse::NotFound().body("File expired or does not exist.")
            }))
    })
    .bind((HOST, PORT.parse().expect("Error converting port to number")))?
    .run()
    .await
}

fn file_helper(file: &TempFile) -> String {
    let mut final_file = String::new();
    if let Some(f) = &file.file_name {
        let filename = generate_random_name(None);
        let split: Vec<_> = f.split(".").collect();
        if split.len() > 1 {
            let extension = split[split.len() - 1];
            final_file = format!("{}.{}", &filename, extension);
        } else {
            final_file = format!("{}", &filename);
        }
    }
    final_file
}
