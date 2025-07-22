#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ExporterBuildError {
    #[error("Invalid URL: {message} ({url:?})")]
    InvalidUrl { url: String, message: String },
    #[error("Exporter backend {name} does not exist")]
    MissingBackend { name: String },
}
