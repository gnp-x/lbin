use actix_files::Files;
use actix_multipart::form::{
    MultipartForm,
    tempfile::{TempFile, TempFileConfig},
};
use actix_web::{App, Error, HttpResponse, HttpServer, Responder, post, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use hightower_naming::generate_random_name;

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(limit = "5MB")]
    file: TempFile,
}

const PORT: &'static str = env!("lbin_port");
const HOST: &'static str = env!("lbin_host");
const AUTH: &'static str = env!("lbin_auth");

#[post("/")]
async fn save_files(
    MultipartForm(form): MultipartForm<UploadForm>,
    cred: BearerAuth,
) -> Result<impl Responder, Error> {
    if cred.token() == AUTH {
        let mut file = String::new();
        let f = form.file;
        let path: String;
        if let Some(s) = f.file_name {
            let filename = generate_random_name(None);
            let split: Vec<_> = s.split(".").collect();
            if split.len() > 1 {
                let extension = split[split.len() - 1];
                path = format!("./tmp/{}.{}", &filename, &extension);
                file = format!("{}.{}\n", &filename, &extension);
            } else {
                path = format!("./tmp/{}", &filename);
                file = format!("{}\n", &filename);
            }
            f.file.persist(&path).ok();
        };
        let url = format!("http://{}:{}/{}", HOST, PORT, &file);
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
            .service(Files::new("/", "./tmp").index_file("../index.html"))
            .default_service(web::to(|| async {
                HttpResponse::NotFound().body("File expired or does not exist.")
            }))
    })
    .bind((HOST, PORT.parse().expect("Error converting port to number")))?
    .run()
    .await
}
