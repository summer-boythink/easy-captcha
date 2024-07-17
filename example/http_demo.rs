use std::{collections::HashMap, fs};

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use easy_captcha::captcha::{rotate::RotateAppState, CaptchaOption, CaptchaType};
use image::ImageFormat;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let images: Vec<String> = fs::read_dir("/root/rs_code/easy-capture/example/test_img")
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            if path.is_file() {
                Some(path.to_str().unwrap().to_string())
            } else {
                None
            }
        })
        .collect();

    let cli = CaptchaOption::new(images, 250, 250, 30.0, CaptchaType::Rotate);

    let app_state = RotateAppState::new(cli, ImageFormat::Jpeg);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .route("/generate", web::get().to(generate_captcha))
            .route("/check", web::post().to(check_captcha))
            .route("/show",  web::get().to(show))
    })
    .bind("0.0.0.0:9666")?
    .run()
    .await
}

#[derive(Deserialize)]
struct CaptchaRequest {
    hash_key: String,
    rotate: f32,
}

#[derive(Serialize)]
struct CaptchaResponse {
    base64_image: String,
    hash_key: String,
}

async fn generate_captcha(data: web::Data<RotateAppState>) -> impl Responder {
    let base64_image = data.generate_captcha();
    let hash_key = data
        .check_map
        .lock()
        .unwrap()
        .keys()
        .last()
        .unwrap()
        .clone();
    HttpResponse::Ok().json(CaptchaResponse {
        base64_image,
        hash_key,
    })
}

async fn check_captcha(
    data: web::Data<RotateAppState>,
    info: web::Json<CaptchaRequest>,
) -> impl Responder {
    let is_valid = data.check_captcha(info.hash_key.clone(), info.rotate);
    HttpResponse::Ok().json(json!({
        "code":200,
        "valid":is_valid
    }))
}

async fn show(data: web::Data<RotateAppState>)-> impl Responder{
    let check_map = data.check_map.lock().unwrap();
    let map_copy: HashMap<String, (f32, f32)> = check_map.clone();
    drop(check_map); // 释放锁

    HttpResponse::Ok().json(json!({
        "code": 200,
        "data": map_copy,
    }))
}