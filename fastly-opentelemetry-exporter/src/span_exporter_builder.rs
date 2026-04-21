use std::str::FromStr;

use fastly::{Backend, http::Url};
use opentelemetry_proto::transform::common::tonic::ResourceAttributesWithSchema;

use crate::{ExporterBuildError, SpanExporter};

/// Builder for configuring a [`SpanExporter`].
///
/// This builder allows you to customize the backend, URL, and resource attributes
/// for the span exporter before building it.
pub struct SpanExporterBuilder {
    backend: Backend,
    resource: Option<ResourceAttributesWithSchema>,
    url: Option<Url>,
}

impl SpanExporterBuilder {
    /// Create a new SpanExporterBuilder that will export spans to a [`fastly::Backend`] with default settings
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

    /// Build the span exporter
    pub fn build(self) -> Result<SpanExporter, ExporterBuildError> {
        let backend = self.backend;

        let resource = self.resource.unwrap_or_default();

        let url = self.url.map_or_else(|| default_url(&backend), Ok)?;

        Ok(SpanExporter::new(backend, resource, url))
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
    /// `/v1/traces`
    pub fn with_url(self, url: Url) -> Self {
        Self {
            url: Some(url),
            ..self
        }
    }
}

fn default_url(backend: &Backend) -> Result<Url, ExporterBuildError> {
    let mut url = Url::from_str(&format!(
        "https://{}:{}/v1/traces",
        backend.get_host(),
        backend.get_port()
    ))
    .unwrap();

    if !backend.is_ssl() {
        url.set_scheme("http").unwrap()
    }

    Ok(url)
}
