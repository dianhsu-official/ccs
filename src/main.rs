use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use std::fs;
use std::path::Path;
mod config;
mod model;
mod template;
use config::SERVER_CONFIG;
#[macro_use]
extern crate lazy_static;
async fn main_post(Json(task): Json<model::Task>) -> StatusCode {
    // create directory for problem
    let path = Path::new(&SERVER_CONFIG.workspace)
        .join(&task.group)
        .join(&task.name);
    let path_str = match path.to_str() {
        Some(path_str) => path_str,
        None => {
            println!("Failed to convert path to string: {:?}", path);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };
    if !path.exists() {
        match fs::create_dir_all(path.clone()) {
            Ok(_) => {}
            Err(_) => {
                println!("Failed to create directory: {}", path_str);
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        }
    }

    // create problem data
    for idx in 0..task.tests.len() {
        let test = &task.tests[idx];
        let input_path = path.join(format!("{:02}.i.txt", idx + 1));
        let output_path = path.join(format!("{:02}.o.txt", idx + 1));
        match fs::write(input_path, &test.input) {
            Ok(_) => {}
            Err(err) => {
                println!("Failed to write input file: {}", err);
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        }
        match fs::write(output_path, &test.output) {
            Ok(_) => {}
            Err(err) => {
                println!("Failed to write output file: {}", err);
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        }
    }
    // create template files
    for template in &SERVER_CONFIG.templates {
        let source_path = Path::new(&template);
        let src_filename = match source_path.file_name() {
            Some(filename) => filename,
            None => {
                println!("Failed to get file name: {:?}", source_path);
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        };
        let destination_path = path.join(src_filename);
        template::handle(source_path, destination_path.clone());
        let destination = match destination_path.to_str() {
            Some(destination) => destination,
            None => {
                println!("Failed to convert path to string: {:?}", destination_path);
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        };
        if SERVER_CONFIG.open_by_vscode {
            #[cfg(target_os = "windows")]
            {
                let mut command = std::process::Command::new("cmd");
                command.arg("/C");
                command.arg("code");
                command.arg(destination);
                match command.spawn() {
                    Ok(_) => {}
                    Err(_) => {
                        println!("Failed to open file by vscode: {}", destination);
                    }
                }
            }
            #[cfg(target_os = "linux")]
            {
                let mut command = std::process::Command::new("code");
                command.arg(destination);
                match command.spawn() {
                    Ok(_) => {}
                    Err(_) => {
                        println!("Failed to open file by vscode: {}", destination);
                    }
                }
            }
        }
    }
    println!("Problem created: {}", path_str);
    return StatusCode::OK;
}
#[tokio::main]
async fn main() {
    println!("Server started.");
    tracing_subscriber::fmt::init();
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/", post(main_post));
    let listener =
        match tokio::net::TcpListener::bind(format!("127.0.0.1:{}", SERVER_CONFIG.port)).await {
            Ok(listener) => listener,
            Err(err) => {
                println!("Failed to bind port: {}", err);
                return;
            }
        };
    match axum::serve(listener, app).await {
        Ok(_) => {
            println!("Server stopped.");
        }
        Err(_) => {
            println!("Server stopped.");
        }
    }
}
