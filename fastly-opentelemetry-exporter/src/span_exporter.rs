use fastly::{
    Backend, Request,
    http::{Method, Url, header::CONTENT_TYPE},
};
use opentelemetry_proto::{
    tonic::collector::trace::v1::ExportTraceServiceRequest,
    transform::{
        common::tonic::ResourceAttributesWithSchema,
        trace::tonic::group_spans_by_resource_and_scope,
    },
};
use opentelemetry_sdk::{
    Resource,
    error::{OTelSdkError, OTelSdkResult},
    trace::SpanData,
};

use crate::{ExporterBuildError, SpanExporterBuilder};

/// An OpenTelemetry trace exporter
#[derive(Debug)]
pub struct SpanExporter {
    backend: Backend,
    resource: ResourceAttributesWithSchema,
    url: Url,
}

impl SpanExporter {
    /// Create a new SpanExporter
    pub fn new(backend: Backend, resource: ResourceAttributesWithSchema, url: Url) -> Self {
        Self {
            backend,
            resource,
            url,
        }
    }

    /// Create a SpanExporterBuilder
    pub fn builder(backend: Backend) -> Result<SpanExporterBuilder, ExporterBuildError> {
        SpanExporterBuilder::new(backend)
    }
}

impl opentelemetry_sdk::trace::SpanExporter for SpanExporter {
    fn set_resource(&mut self, resource: &Resource) {
        self.resource = resource.into();
    }

    fn export(
        &self,
        batch: Vec<SpanData>,
    ) -> impl futures_util::Future<Output = OTelSdkResult> + std::marker::Send {
        let resource_spans = group_spans_by_resource_and_scope(batch, &self.resource);
        let req = ExportTraceServiceRequest { resource_spans };

        let json = match serde_json::to_string(&req) {
            Ok(json) => json,
            Err(e) => {
                return Box::pin(std::future::ready(OTelSdkResult::Err(
                    OTelSdkError::InternalFailure(e.to_string()),
                )));
            }
        };

        let backend = self.backend.clone();
        let url = self.url.clone();

        Box::pin(std::future::ready(send_spans(json, backend, url)))
    }
}

fn send_spans(json: String, backend: Backend, url: Url) -> OTelSdkResult {
    let result = Request::new(Method::POST, url)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body(json)
        .send_async(backend);

    match result {
        Ok(_) => OTelSdkResult::Ok(()),
        Err(e) => OTelSdkResult::Err(OTelSdkError::InternalFailure(e.to_string())),
    }
}
