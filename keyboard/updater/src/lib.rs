mod applier;
mod checker;
mod downloader;
mod model;
mod platform;

pub use applier::apply_update;
pub use checker::check_update;
pub use downloader::download_update;

pub use model::{
    DownloadProgress,
    UpdateInfo,
    UpdateStatus,
};

use std::sync::LazyLock;

static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
});

pub(crate) fn http_client() -> &'static reqwest::Client {
    &HTTP_CLIENT
}