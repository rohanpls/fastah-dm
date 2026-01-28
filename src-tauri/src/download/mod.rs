pub mod manager;
pub mod http;
pub mod gdrive;

use crate::storage::DownloadType;
use tauri::AppHandle;
use crate::download::http::HttpHelper;

pub type DownloadResult<T> = Result<T, DownloadError>;

#[derive(Debug, Clone)]
pub enum DownloadError {
    AccessDenied(String),
    NetworkError(String),
    IoError(String),
    InvalidUrl(String),
    Cancelled,
    ResumeNotPossible(String),
    Other(String),
}

impl std::fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadError::AccessDenied(msg) => write!(f, "Access denied: {}", msg),
            DownloadError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            DownloadError::IoError(msg) => write!(f, "IO error: {}", msg),
            DownloadError::InvalidUrl(msg) => write!(f, "Invalid URL: {}", msg),
            DownloadError::Cancelled => write!(f, "Download cancelled"),
            DownloadError::ResumeNotPossible(msg) => write!(f, "Cannot resume: {}", msg),
            DownloadError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for DownloadError {}

pub struct DownloadContext {
    pub id: String,
    pub url: String,
    pub save_path: String,
    pub app: AppHandle,
    pub http: HttpHelper,
    pub original_url: Option<String>,
    pub downloaded_bytes: u64,
}

pub struct DownloadMeta {
    pub download_type: DownloadType,
    pub direct_url: String,
    pub original_url: Option<String>,
    pub suggested_filename: Option<String>,
}

#[async_trait::async_trait]
pub trait Downloader: Send + Sync {
    fn detect(url: &str) -> bool where Self: Sized;
    
    async fn analyze(url: &str, http: &HttpHelper) -> DownloadResult<Option<DownloadMeta>> 
    where Self: Sized;
    
    async fn run(ctx: DownloadContext) -> DownloadResult<()> where Self: Sized;
    
    async fn refresh_url(original_url: &str, http: &HttpHelper) -> DownloadResult<Option<String>>
    where Self: Sized {
        Ok(None)
    }
}
