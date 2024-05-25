# grpc ease


a Rust crate that provides convenient wrappers and helper structures for working with gRPC in Rust using the Tonic library  
This crate aims to simplify the process of interacting with gRPC services and parsing protocol buffer files.

## Features

- Easy retrieval and parsing of .proto files from servers.
- Helper functions to list gRPC services and RPC methods.

## Installation

Add this to your Cargo.toml:

```yaml
[dependencies]
tonic-helper = "0.1.0"
tonic = "0.11"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
```

## Usage

### Listing Services and RPC Methods

To list all gRPC services and their RPC methods from a server:

```rust
use grpc_ease::reflection::ReflectionClient;

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
```

The `GrpcReflectionClient` struct provides methods to interact with the gRPC reflection API.
- `list_services()`: Retrieves a list of services and their RPC methods.

## License

This project is licensed under the Apache License 2.0. See the [LICENSE](LICENSE.md) file for details.