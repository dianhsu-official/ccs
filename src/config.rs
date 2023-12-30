use clap::Parser;
#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct ServerConfig {
    /// store the workspace path
    #[arg(short, long, default_value = ".")]
    pub workspace: String,

    /// templates: src/main.rs, src/model.rs, src/template.rs
    #[arg(short, long, default_value = "")]
    pub templates: String,

    /// allow open by vscode
    #[arg(short, long, default_value_t = false)]
    pub open_by_vscode: bool,

    /// server port
    #[arg(short, long, default_value_t = 27121)]
    pub port: i32,

    /// verbose mode
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// log to file
    #[arg(short, long, default_value = "stderr")]
    pub log_file: String,
}
lazy_static! {
    pub static ref SERVER_CONFIG: ServerConfig = ServerConfig::parse();
}
