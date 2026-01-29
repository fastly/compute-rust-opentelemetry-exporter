use fastly::{http::StatusCode, Backend, Error, Request, Response};
use fastly_opentelemetry_exporter::*;
use opentelemetry::trace::TracerProvider;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::{
    logs::{SdkLogger, SdkLoggerProvider},
    propagation::TraceContextPropagator,
    trace::{Sampler, SdkTracerProvider, Tracer},
};
use tracing::info;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, Registry};

#[fastly::main]
fn main(req: Request) -> Result<Response, Error> {
    let tracing_backend = Backend::from_name("OTEL-home")?;
    let tracing = Tracing::new(tracing_backend)?;
    tracing.init();

    let _guard = enter_root_span(&req);

    println!(
        "start {}",
        std::env::var("FASTLY_SERVICE_VERSION").unwrap_or("[unknown]".into())
    );

    info!("hello");

    Ok(Response::from_status(StatusCode::OK))
}

struct Tracing {
    logger: SdkLoggerProvider,
    tracer: SdkTracerProvider,
}

impl Tracing {
    fn new(backend: Backend) -> Result<Self, ExporterBuildError> {
        opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

        let log_exporter = LogExporterBuilder::new(backend.clone())?.build()?;

        let resource = ResourceBuilder::build_default();

        let logger = SdkLoggerProvider::builder()
            .with_resource(resource.clone())
            .with_simple_exporter(log_exporter)
            .build();

        let span_exporter = SpanExporterBuilder::new(backend)?.build()?;

        let tracer = SdkTracerProvider::builder()
            .with_resource(resource)
            .with_sampler(Sampler::AlwaysOn)
            .with_simple_exporter(span_exporter)
            .build();

        Ok(Self { logger, tracer })
    }

    fn init(&self) {
        let subscriber = tracing_subscriber::registry()
            .with(self.tracer_layer())
            .with(self.logger_layer());

        tracing::subscriber::set_global_default(subscriber).unwrap();
    }

    fn logger_layer(&self) -> OpenTelemetryTracingBridge<SdkLoggerProvider, SdkLogger> {
        OpenTelemetryTracingBridge::new(&self.logger)
    }

    fn tracer_layer(&self) -> OpenTelemetryLayer<Registry, Tracer> {
        OpenTelemetryLayer::new(self.tracer.tracer(env!("CARGO_PKG_NAME")))
            .with_tracked_inactivity(false)
            .with_threads(false)
    }
}

impl Drop for Tracing {
    fn drop(&mut self) {
        if let Err(e) = self.tracer.shutdown() {
            println!("tracer shutdown failed: {e}");
        }

        if let Err(e) = self.logger.shutdown() {
            println!("logger shutdown failed: {e}");
        }
    }
}
