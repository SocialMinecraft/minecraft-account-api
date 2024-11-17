use anyhow::Result;
use std::env;
use tracing::{Level};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;
pub async fn connect_to_nats() -> Result<async_nats::Client> {
    // Get Nats Env Variable
    let nats_urls_env = match env::var("NATS_URL") {
        Ok(value) => value,
        Err(e) => {
            return Err(anyhow::anyhow!("Couldn't read NATS_URL environment variable: {}", e));
        },
    };

    let nats_urls : Vec<&str> = nats_urls_env.split(",").collect();

    // Connect to NATS server
    let client = async_nats::connect(nats_urls).await?;

    Ok(client)
}

pub fn get_app_name() -> Option<String> {
    env::current_exe()
        .ok()
        .and_then(|pb| pb.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_owned()))
}

pub fn setup_logging(app_name: &str) {
    // Initialize the tracing subscriber with a custom configuration
    tracing_subscriber::fmt()
        // Include thread IDs
        .with_thread_ids(true)
        // Include span events (enter/exit of spans)
        .with_span_events(FmtSpan::FULL)
        // Use a custom environment filter
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive(Level::INFO.into())
                // Add specific module levels
                .add_directive((app_name.to_string()+"=debug").parse().unwrap())
        )
        // Pretty printing for development
        .pretty()
        // Initialize the subscriber
        .init();
}