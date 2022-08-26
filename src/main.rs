use actix_cors::Cors;
use std::io::BufReader;
use std::io::prelude::*;
use actix_files::NamedFile;
use std::path::PathBuf;

use futures_util::stream::StreamExt as _;

use actix_web::http::header::ContentType;
use actix_web::{get, post, web, App, Error, HttpResponse, HttpServer, Responder, Result};
use dotenv;
use migration::{Migrator, MigratorTrait};
use sea_orm::entity::prelude::*;
use sea_orm::{DatabaseConnection, ActiveValue};
use sea_orm::{Database, Set};
use serde::Deserialize;
use serde_json;
use std::env;
use std::fs::File;
use std::fs::remove_file;
use tracing;
use tracing_subscriber;
use futures::{future::ok, stream::once};

use std::io::Write;

// use entities::clip;
mod entities;
use entities::{prelude::*, *};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/edit/{clip_id}")]
async fn get(clip_id: web::Path<i32>, db: web::Data<DatabaseConnection>) -> HttpResponse {
    let clip: Option<clip::Model> = Clip::find_by_id(clip_id.into_inner())
        .one(&*db.into_inner())
        .await
        .unwrap();

    match clip {
        Some(clip) => HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&clip).unwrap()),
        None => {
            HttpResponse::BadRequest().body("noah hat den clip nicht in der datenbank gefunden :(")
        }
    }
}
#[get("/delete/{clip_id}")]
async fn delete(clip_id: web::Path<i32>, db: web::Data<DatabaseConnection>) -> HttpResponse {
    let db = &*db.into_inner();
    let clip_res = Clip::find_by_id(clip_id.into_inner())
        .one(&db.clone())
        .await;
   
    if let Ok(clip) = clip_res {

        let clip: clip::ActiveModel = clip.unwrap().into();
        let video_file_path = clip.streetview_video.clone();
        remove_file(video_file_path.unwrap().unwrap());
        clip.delete(db).await;
        HttpResponse::Ok().body("deleted")
    } else {
        HttpResponse::InternalServerError().body("noahs schuld")
    }
}

#[get("/create_new_clip")]
async fn create_new_clip(db: web::Data<DatabaseConnection>) -> HttpResponse {
    let clip: clip::ActiveModel = Default::default();

    let res = clip.insert(&**db).await;
    match res {
        Ok(res) => HttpResponse::Ok().body(serde_json::to_string(&res).unwrap()),
        _ => HttpResponse::InternalServerError().body("manno"),
    }
}

// Todo: create makro
impl clip::Model {
    fn activate(&self) -> clip::ActiveModel {
        let mut active: clip::ActiveModel = self.clone().into();

        active.g_maps_link = Set(self.g_maps_link.to_owned());
        active.name = Set(self.name.to_owned());
        active.group = Set(self.group.to_owned());
        active.videotext = Set(self.videotext.to_owned());
        active.latlong = Set(self.latlong.to_owned());
        active.start = Set(self.start.to_owned());
        active.stop = Set(self.stop.to_owned());
        // streetview_video is set in send_streetview_video fn
        // active.streetview_video = Set(self.streetview_video.to_owned());
        active.is_renderd = Set(self.is_renderd.to_owned());
        active.is_uploaded_yt = Set(self.is_uploaded_yt.to_owned());
        active.is_uploaded_tik_tok = Set(self.is_uploaded_tik_tok.to_owned());
        active.is_uploaded_instagram = Set(self.is_uploaded_instagram.to_owned());
        active.ymusic_id = Set(self.ymusic_id.to_owned());

        active
    }
}

#[post("/write_edit")]
async fn write_edit(
    payload: web::Json<clip::Model>,
    db: web::Data<DatabaseConnection>,
) -> HttpResponse {
    let old = payload.into_inner();

    let active = old.activate();

    active.update(&**db).await.unwrap();

    HttpResponse::Ok().body("lol")
}

/// wrong name on frontend
#[get("/showall")]
async fn show_all(db: web::Data<DatabaseConnection>) -> HttpResponse {
    let clip_list = Clip::find().all(&**db).await;

    match clip_list {
        Ok(clip_list) => HttpResponse::Ok().body(serde_json::to_string(&clip_list).unwrap()),
        _ => HttpResponse::InternalServerError().body("noaaah"),
    }
}


#[post("/sendStreetviewVideo")]
async fn send_streetview_video(
    // mut content: http::header::ContentDisposition ,
    db: web::Data<DatabaseConnection>,
    mut body: web::Payload,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse, Error> {
    // for demonstration only; in a normal case use the `Bytes` extractor
    // collect payload stream into a bytes object
    // println!("{:?}", content);
    let mut bytes = web::BytesMut::new();
    while let Some(item) = body.next().await {
        bytes.extend_from_slice(&item?);
    }

    let folder_path = env::var("VIDEO_FOLDER").expect("no videofolder path");
    let file_id = req
        .headers()
        .get("Content-Disposition")
        .unwrap()
        .to_str()
        .unwrap();
    let filepath = format!("{folder_path}/streetviewvideo-{file_id}");

    let clip = Clip::find_by_id(file_id.parse::<i32>().unwrap()).one(&**db).await.unwrap().unwrap();

    let mut active: clip::ActiveModel = clip.clone().into();
    active.streetview_video = Set(Some(filepath.clone()));
    active.update(&**db).await.unwrap();


    // TODO: maybe blocking and needs async writing
    let mut f = File::create(filepath).unwrap() ;
    f.write_all(&bytes).unwrap();

    Ok(HttpResponse::Ok().body("stored clip"))
}

async fn get_streetview_video( req: actix_web::HttpRequest) -> Result<NamedFile> {
    let clip_id  = req.match_info().query("filename");
    let folder_path = env::var("VIDEO_FOLDER").expect("no videofolder path");
    let filename = format!("{folder_path}/streetviewvideo-{clip_id}");

    Ok(NamedFile::open(filename)?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .init();

    dotenv::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("no db path");

    let db: DatabaseConnection = Database::connect(&db_url).await.unwrap();
    Migrator::up(&db, None).await.unwrap();

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .app_data(web::Data::new(db.clone()))
            .wrap(cors)
            .service(hello)
            .service(get)
            .service(create_new_clip)
            .service(write_edit)
            .service(delete)
            .service(show_all)
            .service(send_streetview_video)
            .route("/getStreetviewVideo/{filename:.*}.mp4", web::get().to(get_streetview_video))
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
