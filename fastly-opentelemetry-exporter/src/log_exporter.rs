use fastly::{Backend, Request, http::Url};
use opentelemetry_proto::{
    tonic::collector::logs::v1::ExportLogsServiceRequest,
    transform::{
        common::tonic::ResourceAttributesWithSchema, logs::tonic::group_logs_by_resource_and_scope,
    },
};
use opentelemetry_sdk::{
    Resource,
    error::{OTelSdkError, OTelSdkResult},
};

use crate::{ExporterBuildError, log_exporter_builder::LogExporterBuilder};

/// An OpenTelemetry log exporter
#[derive(Debug)]
pub struct LogExporter {
    backend: Backend,
    resource: ResourceAttributesWithSchema,
    url: Url,
}

impl LogExporter {
    /// Create a new LogExporter
    pub fn new(backend: Backend, resource: ResourceAttributesWithSchema, url: Url) -> Self {
        Self {
            backend,
            resource,
            url,
        }
    }

    /// Create a new LogExporterBuilder
    pub fn builder(backend: Backend) -> Result<LogExporterBuilder, ExporterBuildError> {
        LogExporterBuilder::new(backend)
    }
}

impl opentelemetry_sdk::logs::LogExporter for LogExporter {
    async fn export(&self, batch: opentelemetry_sdk::logs::LogBatch<'_>) -> OTelSdkResult {
        let resource_logs = group_logs_by_resource_and_scope(batch, &self.resource);

        let export_request = ExportLogsServiceRequest { resource_logs };

        Request::post(&self.url)
            .with_body_json(&export_request)
            .map_err(|error| OTelSdkError::InternalFailure(error.to_string()))?
            .send_async(&self.backend)
            .map_err(|error| OTelSdkError::InternalFailure(error.to_string()))?;

        Ok(())
    }

    fn set_resource(&mut self, resource: &Resource) {
        self.resource = resource.into();
    }
}
