use crate::trace::TimingMetric;
use std::fmt;

/// Performance metrics for the request-response cycle.
/// See [the spec](https://w3c.github.io/server-timing).
#[derive(Debug, Clone)]
pub struct ServerTiming {
    /// Server timing metrics.
    metrics: Vec<TimingMetric>,
}

impl ServerTiming {
    /// Creates a new instance.
    #[inline]
    pub fn new() -> Self {
        Self {
            metrics: Vec::new(),
        }
    }

    /// Pushes an entry into the list of metrics.
    #[inline]
    pub fn push(&mut self, metric: TimingMetric) {
        self.metrics.push(metric);
    }
}

impl Default for ServerTiming {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ServerTiming {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output = self
            .metrics
            .iter()
            .map(|metric| metric.to_string())
            .intersperse(", ".to_string())
            .collect::<String>();
        write!(f, "{output}")
    }
}