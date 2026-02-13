use std::time::Duration;

use fastly::{
    Backend, Request,
    http::{HeaderName, HeaderValue, Url},
};
use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
use opentelemetry_sdk::{
    error::{OTelSdkError, OTelSdkResult},
    metrics::{Temporality, data::ResourceMetrics, exporter::PushMetricExporter},
};

#[derive(Debug)]
pub struct MetricsExporter {
    backend: Backend,
    headers: Vec<(HeaderName, HeaderValue)>,
    temporality: Temporality,
    url: Url,
}

impl MetricsExporter {
    pub fn new(
        backend: Backend,
        headers: Vec<(HeaderName, HeaderValue)>,
        temporality: Temporality,
        url: Url,
    ) -> Self {
        Self {
            backend,
            headers,
            temporality,
            url,
        }
    }
}

impl PushMetricExporter for MetricsExporter {
    async fn export(&self, metrics: &ResourceMetrics) -> OTelSdkResult {
        let export_request: ExportMetricsServiceRequest = metrics.into();
        println!("export: {:#?}", export_request);

        let mut request = Request::post(&self.url)
            .with_body_json(&export_request)
            .map_err(|error| OTelSdkError::InternalFailure(error.to_string()))?;

        for (name, value) in &self.headers {
            request.append_header(name, value);
        }

        let res = request
            .send(&self.backend)
            .map_err(|error| OTelSdkError::InternalFailure(error.to_string()))?;

        println!("export response: {}", res.get_status());

        Ok(())
    }

    fn force_flush(&self) -> OTelSdkResult {
        println!("force flush");
        Ok(())
    }

    fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
        println!("shutdown with timeout");
        Ok(())
    }

    fn temporality(&self) -> Temporality {
        self.temporality
    }
}
