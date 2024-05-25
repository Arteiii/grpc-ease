use grpc_ease::reflection::ReflectionClient;
use tonic::transport::Channel;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = "http://[::1]:4444";

    let mut reflection_client = ReflectionClient::new(endpoint.to_string()).await?;
    let services = reflection_client.list_services().await?;

    for service in services {
        println!("Service: {}", service.service);
        println!("Package: {}", service.package);
        for method in service.methods {
            println!("  RPC Method: {}", method.name);
        }
    }

    Ok(())
}