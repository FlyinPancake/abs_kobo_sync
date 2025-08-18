// Domain models device-agnostic to map ABS entities into something our API returns

#[derive(Debug, Clone)]
pub struct SeriesRef {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum FileKind {
    Epub,
    Pdf,
    M4b,
    Mp3,
    Unknown(String),
}

#[derive(Debug, Clone)]
pub struct FileRef {
    pub kind: FileKind,
    pub url: String,
    pub size: Option<u64>,
    pub mime: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Book {
    pub id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub series: Option<SeriesRef>,
    pub cover_url: Option<String>,
    pub formats: Vec<FileRef>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Progress {
    pub book_id: String,
    pub device_id: String,
    /// 0.0 - 1.0 fraction
    pub position: f64,
    pub updated_at_epoch_ms: i64,
}
