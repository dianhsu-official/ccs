use dotenvy::dotenv;
use std::fs;
use std::path::Path;
use std::{
    env,
    io::Read,
    net::{TcpListener, TcpStream},
};
mod model;
fn handle(mut stream: TcpStream, workspace: &str, templates: &Vec<String>, open_by_vscode: bool) {
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
            process(&request[pos..], workspace, &templates, open_by_vscode);
        }
    }
}
fn process(request: &str, workspace: &str, templates: &Vec<String>, open_by_vscode: bool) {
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
        match fs::create_dir_all(path.clone()) {
            Ok(_) => {},
            Err(_) => {},
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
    // create template files
    for template in templates {
        let source_path = Path::new(template);
        let destination_path = path.join(source_path.file_name().unwrap());
        if !destination_path.exists() {
            fs::copy(source_path, destination_path.clone()).unwrap();
        }
        let destination = destination_path.to_str().unwrap();
        if open_by_vscode {
            #[cfg(target_os = "windows")]
            {
                let mut command = std::process::Command::new("cmd");
                command.arg("/C");
                command.arg("code");
                command.arg(destination);
                command.spawn().unwrap();
            }
            #[cfg(target_os = "linux")]
            {
                let mut command = std::process::Command::new("code");
                command.arg(destination);
                command.spawn().unwrap();
            }
        }
    }
    println!("Problem created: {}", path.to_str().unwrap());
}
fn main() {
    dotenv().ok();
    println!("Server started.");
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
    let open_by_vscode: bool = match env::var("OPEN_BY_VSCODE") {
        Ok(open_by_vscode) => open_by_vscode == "true",
        Err(_) => false,
    };
    if templates.is_empty() {
        println!("No templates found.");
        return;
    }
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle(stream, &workspace, &templates, open_by_vscode);
            }
            Err(e) => {
                println!("Unable to connect: {}", e);
            }
        }
    }
}
