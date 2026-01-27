use reqwest::header::{CONTENT_LENGTH, ETAG, ACCEPT_RANGES, RANGE};
use reqwest::Client;

#[derive(Clone)]
pub struct HttpHelper {
    client: Client,
}

#[derive(Debug, Clone)]
pub struct DownloadMetadata {
    pub size: Option<u64>,
    pub etag: Option<String>,
    pub accept_ranges: bool,
}

impl HttpHelper {
    pub fn new() -> Self {
        Self {
            client: Client::builder().build().unwrap(), // Default client
        }
    }

    pub async fn get_metadata(&self, url: &str) -> Result<DownloadMetadata, String> {
        let res = self.client.head(url).send().await.map_err(|e| e.to_string())?; 
        // Note: Some servers might not support HEAD properly or return different headers. 
        // A GET with Range: bytes=0-0 might be safer fallback, but sticking to HEAD for now.
        
        let headers = res.headers();

        let size = headers.get(CONTENT_LENGTH)
            .and_then(|val| val.to_str().ok())
            .and_then(|val| val.parse::<u64>().ok());
        
        let etag = headers.get(ETAG)
            .and_then(|val| val.to_str().ok())
            .map(|s| s.to_string());

        let accept_ranges = headers.get(ACCEPT_RANGES)
            .and_then(|val| val.to_str().ok())
            .map(|s| s == "bytes")
            .unwrap_or(false);

        Ok(DownloadMetadata {
            size,
            etag,
            accept_ranges,
        })
    }

    pub async fn download_range_request(&self, url: &str, start: u64, end: u64) -> Result<reqwest::Response, String> {
        let range_header = format!("bytes={}-{}", start, end);
        self.client.get(url)
            .header(RANGE, range_header)
            .send()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn download_stream_request(&self, url: &str) -> Result<reqwest::Response, String> {
        self.client.get(url)
            .send()
            .await
            .map_err(|e| e.to_string())
    }
}
