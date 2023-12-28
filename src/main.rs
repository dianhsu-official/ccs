use dotenvy::dotenv;
use std::fs;
use std::path::Path;
use std::{
    env,
    io::Read,
    net::{TcpListener, TcpStream},
};
mod model;
fn handle(mut stream: TcpStream, workspace: &str, templates: &Vec<String>) {
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).unwrap();
    let request = String::from_utf8_lossy(&buf[..]).to_string();
    if request.is_empty() {
        return;
    }
    let pos = request.find('{');
    match pos {
        None => {
            println!("Bad request: {}", request);
        }
        Some(pos) => {
            process(&request[pos..], workspace, &templates);
        }
    }
}
fn process(request: &str, workspace: &str, templates: &Vec<String>) {
    // read problem data from request
    let task = match serde_json::from_str::<model::Task>(request) {
        Ok(task) => task,
        Err(_) => {
            println!("Bad request: {}", request);
            return;
        }
    };

    // create directory for problem
    let path = Path::new(workspace).join(&task.group).join(&task.name);
    if !path.exists() {
        fs::create_dir_all(path.clone()).unwrap();
    }

    // create template files
    for template in templates {
        let source_path = Path::new(template);
        let destination_path = path.join(source_path.file_name().unwrap());
        if !destination_path.exists() {
            fs::copy(source_path, destination_path).unwrap();
        }
    }

    // create problem data
    for idx in 0..task.tests.len() {
        let test = &task.tests[idx];
        let input_path = path.join(format!("{:02}.i.txt", idx + 1));
        let output_path = path.join(format!("{:02}.o.txt", idx + 1));
        fs::write(input_path, &test.input).unwrap();
        fs::write(output_path, &test.output).unwrap();
    }
}
fn main() {
    dotenv().ok();
    let port = env::var("PORT").unwrap_or_else(|_| "27121".to_string());
    let workspace = match env::var("WORKSPACE") {
        Ok(workspace) => workspace,
        Err(_) => {
            println!("WORKSPACE not set.");
            return;
        }
    };
    let templates = match env::var("TEMPLATES") {
        Ok(templates) => templates
            .split(',')
            .into_iter()
            .map(|s| s.to_string().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>(),
        Err(_) => {
            println!("TEMPLATES not set.");
            return;
        }
    };
    if templates.is_empty() {
        println!("No templates found.");
        return;
    }
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle(stream, &workspace, &templates);
            }
            Err(e) => {
                println!("Unable to connect: {}", e);
            }
        }
    }
}
