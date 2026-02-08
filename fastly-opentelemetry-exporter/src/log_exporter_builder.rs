use fastly::{
    Backend,
    http::{HeaderName, HeaderValue, Url},
};
use opentelemetry_proto::transform::common::tonic::ResourceAttributesWithSchema;

use crate::{DefaultUrl, ExporterBuildError, LogExporter};

pub struct LogExporterBuilder {
    backend: Backend,
    headers: Vec<(HeaderName, HeaderValue)>,
    resource: Option<ResourceAttributesWithSchema>,
    url: Option<Url>,
}

impl LogExporterBuilder {
    /// Create a new LogExporterBuilder that will export logs to a [`fastly::Backend`] with default
    /// settings
    pub fn new(backend: Backend) -> Result<Self, ExporterBuildError> {
        if !backend.exists() {
            return Err(ExporterBuildError::MissingBackend {
                name: backend.name().into(),
            });
        }

        Ok(Self {
            backend,
            headers: Default::default(),
            resource: None,
            url: None,
        })
    }

    /// Build the log exporter
    pub fn build(self) -> Result<LogExporter, ExporterBuildError> {
        let Self {
            backend,
            headers,
            resource,
            url,
        } = self;

        let resource = resource.unwrap_or_default();

        let url = url.map_or_else(|| DefaultUrl::Logs.to_url(&backend), Ok)?;

        Ok(LogExporter::new(backend, headers, resource, url))
    }

    /// Add a header
    pub fn with_header(mut self, name: HeaderName, value: HeaderValue) -> Self {
        self.headers.push((name, value));

        self
    }

    /// Override the exporter resource
    pub fn with_resource(self, resource: ResourceAttributesWithSchema) -> Self {
        Self {
            resource: Some(resource),
            ..self
        }
    }

    /// Override the exporter URL
    ///
    /// If the URL is left unspecified it will be constructed from the backend and export traces to
    /// `/v1/logs`
    pub fn with_url(self, url: Url) -> Self {
        Self {
            url: Some(url),
            ..self
        }
    }
}
