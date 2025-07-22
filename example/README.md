# Fastly OpenTelemetry Exporter Example

You must create a backend named `otel-http` that points to an OpenTelemetry
HTTP endpoint to use this service.

Making a request to this service will return an empty OK response.  A trace
will be sent to your backend.
