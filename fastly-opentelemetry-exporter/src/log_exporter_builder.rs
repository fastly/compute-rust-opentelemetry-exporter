use fastly::{Backend, http::Url};
use opentelemetry_proto::transform::common::tonic::ResourceAttributesWithSchema;

use crate::{DefaultUrl, ExporterBuildError, LogExporter};

pub struct LogExporterBuilder {
    backend: Backend,
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
            resource: None,
            url: None,
        })
    }

    /// Build the log exporter
    pub fn build(self) -> Result<LogExporter, ExporterBuildError> {
        let backend = self.backend;

        let resource = self.resource.unwrap_or_default();

        let url = self
            .url
            .map_or_else(|| DefaultUrl::Logs.to_url(&backend), Ok)?;

        Ok(LogExporter::new(backend, resource, url))
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
