use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use std::fs::File;
use std::path::Path;
use tokio::{fs, signal};
mod config;
mod model;
mod template;
use config::SERVER_CONFIG;
use log;
use log::LevelFilter;
use std::io::Write;
#[macro_use]
extern crate lazy_static;
const ILLEGAL_CHARS: [char; 10] = ['\\', '/', ':', '*', '?', '"', '<', '>', '|', '\0'];
async fn get_path(task: &model::Task) -> Result<String, String> {
    let mut groups: Vec<String> = Vec::new();
    if SERVER_CONFIG.short_path {
        if task.group.starts_with("Codeforces") {
            // match problem url: https://<any domain>/contest/1461/problem/A
            let urls = task.url.split('/').into_iter().collect::<Vec<&str>>();
            let problem = urls[urls.len() - 1].to_string();
            let contest = urls[urls.len() - 3].to_string();
            groups.push("Codeforces".to_string());
            groups.push(contest);
            groups.push(problem);
        } else if task.group.starts_with("AtCoder") {
            // match problem url: https://atcoder.jp/contests/abc335/tasks/abc335_a
            if let Some(contest_problem) = task.url.split('/').into_iter().last() {
                groups.push("AtCoder".to_string());
                let contest_problem = contest_problem.to_string();
                let contest = contest_problem.split('_').nth(0).unwrap().to_string();
                let problem = contest_problem.split('_').nth(1).unwrap().to_string();
                groups.push(contest);
                groups.push(problem);
            }
        }
    }
    if groups.is_empty() {
        let mut contest = task.group.clone();
        for illegal_char in ILLEGAL_CHARS.iter() {
            contest = contest.replace(*illegal_char, "_");
        }
        groups.push(contest);
        let problem = task.name.clone();
        groups.push(problem);
    }
    log::debug!("Problem groups: {:?}", groups);
    let mut path = Path::new(&SERVER_CONFIG.workspace).to_path_buf();
    for group in groups {
        path = path.join(group);
    }
    let path_str = match path.to_str() {
        Some(path_str) => path_str,
        None => {
            return Err("Failed to convert path to string.".to_string());
        }
    };
    log::debug!("Problem path: {}", path_str);
    return Ok(path_str.to_string());
}

async fn main_post(Json(task): Json<model::Task>) -> StatusCode {
    log::debug!("Problem received: {:?}", task);
    let path_str = match get_path(&task).await {
        Ok(path_str) => path_str,
        Err(info) => {
            log::error!("{}", info);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };
    let path = Path::new(&path_str);
    if !path.exists() {
        match fs::create_dir_all(path).await {
            Ok(_) => {}
            Err(info) => {
                log::error!("Failed to create directory: {}, {}", path_str, info);
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
        // handle template with line number and column number: src/main.rs:<line>:<column>
        let template_file = template.split(":").into_iter().collect::<Vec<&str>>();
        let source_path = Path::new(&template_file[0]);
        let src_filename = match source_path.file_name() {
            Some(filename) => filename,
            None => {
                log::error!("Failed to get file name: {:?}", source_path);
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        };
        let destination_pathbuf = path.join(src_filename);
        let destination_path = destination_pathbuf.as_path();
        let destination = match destination_path.to_str() {
            Some(destination) => destination,
            None => {
                log::error!("Failed to convert path to string: {:?}", destination_path);
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        };
        template::handle(source_path, destination_path);
        if SERVER_CONFIG.open_by_vscode {
            let mut open_path = destination.to_string();
            for idx in 1..template_file.len() {
                open_path.push_str(&format!(":{}", template_file[idx]));
            }
            log::debug!("Opening file by vscode: {}", open_path);
            #[cfg(target_os = "windows")]
            {
                let mut command = std::process::Command::new("powershell");
                command.arg("-c");
                command.arg("code");
                command.arg("-g");
                command.arg(format!("\"{}\"", open_path));
                match command.spawn() {
                    Ok(_) => {}
                    Err(_) => {
                        log::error!("Failed to open file by vscode: {}", open_path);
                    }
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                let mut command = std::process::Command::new("code");
                command.arg("-g");
                command.arg(format!("\"{}\"", open_path));
                match command.spawn() {
                    Ok(_) => {}
                    Err(_) => {
                        log::error!("Failed to open file by vscode: {}", open_path);
                    }
                }
            }
        }
    }
    log::debug!("Problem created: {}", path_str);
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
                LevelFilter::Info
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
    match axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        Ok(_) => {
            log::info!("Server stopped.");
        }
        Err(_) => {
            log::error!("Server stopped with error.");
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {
        },
        _ = terminate => {},
    }
}
