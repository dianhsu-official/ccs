use std::env;

use clap::Parser;
use dotenvy::dotenv;

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct ServerConfig {

    /// store the workspace path
    #[arg(short, long, default_value = "")]
    pub workspace: String,

    /// templates: src/main.rs, src/model.rs, src/template.rs
    #[arg(short, long, default_value = "")]
    pub templates: Vec<String>,

    /// allow open by vscode
    #[arg(short, long, default_value = "false")]
    pub open_by_vscode: bool,

    /// server port
    #[arg(short, long, default_value = "27121")]
    pub port: i32,
}
lazy_static! {
    pub static ref SERVER_CONFIG: ServerConfig = {
        dotenv().ok();
        let mut args = ServerConfig::parse();
        if args.workspace.is_empty() {
            args.workspace = match env::var("WORKSPACE") {
                Ok(workspace) => workspace,
                Err(_) => String::new(),
            };
        }
        if args.templates.is_empty() {
            args.templates = match env::var("TEMPLATES") {
                Ok(templates) => templates
                    .split(',')
                    .into_iter()
                    .map(|s| s.to_string().trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<String>>(),
                Err(_) => Vec::new(),
            };
        }
        if args.open_by_vscode == false {
            args.open_by_vscode = match env::var("OPEN_BY_VSCODE") {
                Ok(open_by_vscode) => open_by_vscode == "true",
                Err(_) => false,
            };
        }
        if args.port == 27121 {
            args.port = match env::var("PORT") {
                Ok(port) => port.parse::<i32>().unwrap_or(27121),
                Err(_) => 27121,
            };
        }
        println!("Server config: {:?}", args);
        args
    };
}
