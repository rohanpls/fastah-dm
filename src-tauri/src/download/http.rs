use reqwest::header::{CONTENT_LENGTH, ETAG, ACCEPT_RANGES, RANGE};
use reqwest::Client;

#[derive(Clone)]
pub struct HttpHelper {
    client: Client,
}

#[derive(Debug, Clone)]
pub struct DownloadMetadata {
    pub size: Option<u64>,
    #[allow(dead_code)]
    pub etag: Option<String>,
    pub accept_ranges: bool,
}

impl HttpHelper {
    pub fn new() -> Self {
        let client = Client::builder()
            .cookie_store(true)
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .unwrap_or_else(|_| Client::new());
        
        Self { client }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub async fn get_metadata(&self, url: &str) -> Result<DownloadMetadata, String> {
        // Try HEAD request, but don't fail if server doesn't support it
        // Some servers (like direct file download servers) return 405 or fail on HEAD
        match self.client.head(url).send().await {
            Ok(res) => {
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
            },
            Err(_) => {
                // Server doesn't support HEAD, return empty metadata
                // This will result in indeterminate progress (no size info)
                Ok(DownloadMetadata {
                    size: None,
                    etag: None,
                    accept_ranges: false,
                })
            }
        }
    }

    pub async fn download_range_request(&self, url: &str, start: u64, _end: u64) -> Result<reqwest::Response, String> {
        // Use open-ended range format "bytes=start-" for proper resume support
        let range_header = format!("bytes={}-", start);
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
