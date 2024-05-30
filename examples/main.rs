use grpc_ease::reflection::ReflectionClient;
use prost::bytes::Bytes;
use std::error::Error;
use std::io;
use std::io::Write;
use tonic::codegen::{http, Body, StdError};
use tonic::{GrpcMethod, Status};

macro_rules! init_tracing {
    ($env_var:expr) => {{
        use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

        let env_filter = if let Ok(env) = std::env::var($env_var) {
            EnvFilter::new(env)
        } else if cfg!(debug_assertions) {
            EnvFilter::new(tracing::Level::DEBUG.to_string())
        } else {
            EnvFilter::new(tracing::Level::INFO.to_string())
        };

        tracing_subscriber::registry()
            .with(fmt::layer())
            .with(env_filter)
            .init();

        if cfg!(debug_assertions) {
            tracing::error!("This is an error message");
            tracing::warn!("This is a warning message");
            tracing::info!("This is an info message");
            tracing::debug!("This is a debug message");
            tracing::trace!("This is a trace message");
        }
    }};
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let reflection_client = ReflectionClient::new("http://0.0.0.0:6666".to_string()).await?;

    cli_loop(reflection_client).await.expect("cli panic");

    Ok(())
}


async fn cli_loop(mut reflection_client: ReflectionClient) -> Result<(), Box<dyn Error>> {
    loop {
        print!("Enter command: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim();

        if input.eq_ignore_ascii_case("list services") {
            // List services
            match reflection_client.list_services().await {
                Ok(services) => {
                    for service in services {
                        println!("Service: {}", service.service);
                        println!("Package: {}", service.package);
                        for method in service.methods {
                            println!("  RPC Method: {}", method.name);
                            println!("      Request: {}", method.request);
                            println!("      Response: {}", method.response);
                        }
                    }
                }
                Err(err) => {
                    println!("Error listing services: {}", err);
                }
            }
        } else if input.eq_ignore_ascii_case("exit") {
            break Ok(());
        } else {
            println!("Unrecognized command.");
        }
    }
}
