use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    ShowResults(PathBuf),
}
