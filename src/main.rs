use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use std::fs::File;
use std::path::Path;
use tokio::fs;
mod config;
mod model;
mod template;
use config::SERVER_CONFIG;
use log;
use log::LevelFilter;
use std::io::Write;
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
            log::error!("Failed to convert path to string: {:?}", path);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };
    if !path.exists() {
        match fs::create_dir_all(path.clone()).await {
            Ok(_) => {}
            Err(_) => {
                log::error!("Failed to create directory: {}", path_str);
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        }
    }

    // create problem data
    for idx in 0..task.tests.len() {
        let test = &task.tests[idx];
        let input_path = path.join(format!("{:02}.i.txt", idx + 1));
        let output_path = path.join(format!("{:02}.o.txt", idx + 1));
        match fs::write(input_path, &test.input).await {
            Ok(_) => {}
            Err(err) => {
                log::error!("Failed to write input file: {}", err);
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        }
        match fs::write(output_path, &test.output).await {
            Ok(_) => {}
            Err(err) => {
                log::error!("Failed to write output file: {}", err);
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        }
    }
    // create template files
    let templates: Vec<&str> = SERVER_CONFIG
        .templates
        .split(',')
        .into_iter()
        .filter_map(|x| {
            if x.trim().is_empty() {
                None
            } else {
                Some(x.trim())
            }
        })
        .collect();
    for template in templates {
        let source_path = Path::new(&template);
        let src_filename = match source_path.file_name() {
            Some(filename) => filename,
            None => {
                log::error!("Failed to get file name: {:?}", source_path);
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        };
        let destination_path = path.join(src_filename);
        template::handle(source_path, destination_path.clone());
        let destination = match destination_path.to_str() {
            Some(destination) => destination,
            None => {
                log::error!("Failed to convert path to string: {:?}", destination_path);
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
                        log::error!("Failed to open file by vscode: {}", destination);
                    }
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                let mut command = std::process::Command::new("code");
                command.arg(destination);
                match command.spawn() {
                    Ok(_) => {}
                    Err(_) => {
                        log::error!("Failed to open file by vscode: {}", destination);
                    }
                }
            }
        }
    }
    log::info!("Problem created: {}", path_str);
    return StatusCode::OK;
}
#[tokio::main]
async fn main() {
    // setup logs
    let mut builder = env_logger::builder();
    builder
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {}:{}] [{}] - {}",
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.level(),
                record.args()
            )
        })
        .filter(
            Some("ccs"),
            if SERVER_CONFIG.verbose {
                LevelFilter::Debug
            } else {
                LevelFilter::Warn
            },
        )
        .write_style(env_logger::WriteStyle::Auto);
    match SERVER_CONFIG.log_file.as_str() {
        "stderr" => {}
        "stdout" => {
            builder.target(env_logger::Target::Stdout);
        }
        file => {
            let target = Box::new(File::create(file).expect("Failed to create log file."));
            builder.target(env_logger::Target::Pipe(target));
        }
    }
    builder.init();

    // start server
    log::info!("Server started.");
    let app = Router::new()
        .route("/", get(|| async { "Hello, CCS!" }))
        .route("/", post(main_post));
    let listener =
        match tokio::net::TcpListener::bind(format!("127.0.0.1:{}", SERVER_CONFIG.port)).await {
            Ok(listener) => listener,
            Err(err) => {
                log::error!("Failed to bind port: {}", err);
                return;
            }
        };
    match axum::serve(listener, app).await {
        Ok(_) => {
            log::info!("Server stopped.");
        }
        Err(err) => {
            log::error!("Failed to serve: {}", err);
        }
    }
}
