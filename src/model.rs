use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub enum IOEnum {
    #[serde(rename = "stdin")]
    StdIn,
    #[serde(rename = "stdout")]
    StdOut,
    #[serde(rename = "file")]
    File,
    #[serde(rename = "regex")]
    Regex,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct IOType {
    #[serde(rename = "type")]
    pub io_type: IOEnum,
    #[serde(rename = "fileName")]
    pub file_name: Option<String>,
    pub pattern: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Test {
    pub input: String,
    pub output: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TaskClass {
    #[serde(rename = "taskClass")]
    pub task_class: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Languages {
    pub java: TaskClass,
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
pub struct Task {
    pub name: String,
    pub group: String,
    pub url: String,
    pub interactive: bool,
    #[serde(rename = "timeLimit")]
    pub time_limit: u64,
    pub tests: Vec<Test>,
    #[serde(rename = "testType")]
    pub test_type: String,
    pub input: IOType,
    pub output: IOType,
    pub languages: Languages,
}
