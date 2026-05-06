use fastly::{Backend, Error, Request, Response, http::StatusCode};
use fastly_opentelemetry_exporter::*;
use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::{
    propagation::TraceContextPropagator,
    trace::{Sampler, SdkTracerProvider, Tracer},
};
use tracing::info;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{Registry, layer::SubscriberExt};

#[fastly::main]
fn main(req: Request) -> Result<Response, Error> {
    let tracing_backend = Backend::from_name("otel-http")?;
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
    tracer: SdkTracerProvider,
}

impl Tracing {
    fn new(backend: Backend) -> Result<Self, ExporterBuildError> {
        opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

        let exporter = SpanExporterBuilder::new(backend)?.build()?;

        let tracer = SdkTracerProvider::builder()
            .with_resource(ResourceBuilder::build_default())
            .with_sampler(Sampler::AlwaysOn)
            .with_simple_exporter(exporter)
            .build();

        Ok(Self { tracer })
    }

    fn init(&self) {
        let subscriber = tracing_subscriber::registry().with(self.layer());

        tracing::subscriber::set_global_default(subscriber).unwrap();
    }

    fn layer(&self) -> OpenTelemetryLayer<Registry, Tracer> {
        OpenTelemetryLayer::new(self.tracer.tracer(env!("CARGO_PKG_NAME")))
            .with_tracked_inactivity(false)
            .with_threads(false)
    }
}

impl Drop for Tracing {
    fn drop(&mut self) {
        if let Err(e) = self.tracer.shutdown() {
            println!("shutdown failed: {e}");
        }
    }
}
