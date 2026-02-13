use std::str::FromStr;

use fastly::{Backend, http::Url};

use crate::ExporterBuildError;

#[derive(Clone, Copy)]
pub(crate) enum DefaultUrl {
    Logs,
    Metrics,
    Traces,
}

impl DefaultUrl {
    pub(crate) fn to_url(&self, backend: &Backend) -> Result<Url, ExporterBuildError> {
        let scheme = if backend.is_ssl() { "https" } else { "http" };

        let path = match self {
            DefaultUrl::Logs => "/v1/logs",
            DefaultUrl::Metrics => "/v1/metrics",
            DefaultUrl::Traces => "/v1/traces",
        };

        let url_string = format!(
            "{scheme}://{}:{}{path}",
            backend.get_host(),
            backend.get_port()
        );

        Url::from_str(&url_string).map_err(|error| ExporterBuildError::InvalidUrl {
            url: url_string,
            message: error.to_string(),
        })
    }
}
