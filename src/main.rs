use actix_files::Files;
use actix_multipart::form::{
    MultipartForm,
    tempfile::{TempFile, TempFileConfig},
};
use actix_web::{App, Error, HttpResponse, HttpServer, Responder, post};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use hightower_naming::generate_random_name;

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(rename = "file", limit = "5MB")]
    file: TempFile,
}

const PORT: &'static str = env!("port");
const HOST: &'static str = env!("host");
const AUTH: &'static str = env!("auth");

#[post("/")]
async fn save_files(
    MultipartForm(form): MultipartForm<UploadForm>,
    cred: BearerAuth,
) -> Result<impl Responder, Error> {
    let mut file = String::new();
    if cred.token() == AUTH {
        let f = form.file;
        let path: String;
        if let Some(s) = f.file_name {
            let filename = generate_random_name(None);
            let split: Vec<_> = s.split(".").collect();
            if split.len() > 1 {
                let extension = split[split.len() - 1];
                path = format!("./tmp/{}.{}", filename, extension);
                file.push_str(&filename);
                file.push_str(".");
                file.push_str(&extension);
            } else {
                path = format!("./tmp/{}", filename);
                file.push_str(&filename);
            }
            f.file.persist(&path).ok();
        };
        file.push('\n');
        let url = format!("http://{}:{}/{}", HOST, PORT, file);
        Ok(HttpResponse::Ok().body(url))
    } else {
        Ok(HttpResponse::Unauthorized().body("Invalid auth token.\n"))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::fs::create_dir_all("./tmp")?;
    println!("Starting up file server on port {HOST}:{PORT}");
    HttpServer::new(|| {
        App::new()
            .app_data(TempFileConfig::default().directory("./tmp"))
            .service(save_files)
            .service(Files::new("/", "./tmp").index_file("index.html"))
    })
    .bind((HOST, PORT.parse().expect("Error converting port to number")))?
    .run()
    .await
}
