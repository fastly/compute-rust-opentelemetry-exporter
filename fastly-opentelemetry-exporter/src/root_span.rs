use fastly::Request;
use opentelemetry::{propagation::Extractor, trace::TraceContextExt};
use tracing::{Span, info_span, span::EnteredSpan};
use tracing_opentelemetry::{OpenTelemetrySpanExt, SetParentError};

pub fn enter_root_span(req: &Request) -> Result<EnteredSpan, SetParentError> {
    let span = info_span!("root");

    update_span_for_request(&span, req)?;

    Ok(span.entered())
}

struct HeaderExtractor<'a>(pub &'a Request);

impl Extractor for HeaderExtractor<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get_header(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.get_header_names().map(|n| n.as_str()).collect()
    }
}

pub fn update_span_for_request(span: &Span, req: &Request) -> Result<String, SetParentError> {
    let cx = opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(req))
    });

    span.set_parent(cx)?;
    Ok(span.context().span().span_context().trace_id().to_string())
}
