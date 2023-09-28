//! I define few utilities to setup tracing.
//!

use tracing::{subscriber::set_global_default, Subscriber};
use tracing_log::LogTracer;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

/// Get a simple subscriber.
pub fn get_subscriber(_name: String, env_filter: String) -> impl Subscriber + Send + Sync {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = fmt::Layer::default().pretty();
    Registry::default().with(env_filter).with(formatting_layer)
}

/// Register a subscriber as global default to process span data.
///
/// It should only be called once!
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}
