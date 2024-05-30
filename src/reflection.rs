use crate::service_info::{MethodInfo, ServiceInfo};
use prost::Message;
use serde_json::Value;
use std::error::Error;
use tokio_stream::StreamExt;
use tonic::{transport::Channel, Request, Response};
use tonic_reflection::pb::{
    server_reflection_client::ServerReflectionClient, server_reflection_request::MessageRequest,
    server_reflection_response::MessageResponse, ServerReflectionRequest,
};
use tracing::{debug, trace};

pub struct ReflectionClient {
    client: ServerReflectionClient<Channel>,
}

impl ReflectionClient {
    /// Creates a new instance of the client, connecting to the specified endpoint.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - A `String` containing the server endpoint URL.
    ///
    /// # Returns
    ///
    /// * `Result<Self, Box<dyn Error>>` - A result containing the newly created client instance
    ///   or an error if the connection fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if the endpoint URL is invalid or if the connection
    /// to the server cannot be established.
    ///
    /// # Example
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// let client = grpc_ease::reflection::ReflectionClient::new("http://localhost:50051".to_string()).await?;
    /// # })
    /// ```
    pub async fn new(endpoint: String) -> Result<Self, Box<dyn Error>> {
        let channel = Channel::from_shared(endpoint)?.connect().await?;
        Ok(Self {
            client: ServerReflectionClient::new(channel),
        })
    }

    async fn make_request(
        &mut self,
        request: ServerReflectionRequest,
    ) -> Result<MessageResponse, Box<dyn Error>> {
        let request = Request::new(tokio_stream::once(request));
        let mut inbound = self
            .client
            .server_reflection_info(request)
            .await?
            .into_inner();

        if let Some(response) = inbound.next().await {
            return Ok(response?.message_response.expect("some MessageResponse"));
        }

        Err("No response received".into())
    }

    /// Retrieves a list of services available on the server along with their methods.
    ///
    /// This function sends a `ServerReflectionRequest` to the server to list all available services.
    /// For each service, it fetches the `FileDescriptorProto` to gather detailed information about the
    /// service, including its methods.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<ServiceInfo>, Box<dyn Error>>` - A result containing a vector of `ServiceInfo`
    ///   objects, each representing a service and its methods, if the request is successful
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The request to the server fails
    /// - The response from the server is not a `ListServicesResponse`
    /// - The file descriptors cannot be retrieved or decoded
    /// - The service or method names are missing in the descriptors
    ///
    /// # Example
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// let mut client = grpc_ease::reflection::ReflectionClient::connect("http://localhost:50051").await?;
    /// let services = client.list_services().await?;
    /// for service in services {
    ///     println!("Service: {}.{}", service.package, service.service);
    ///     for method in service.methods {
    ///         println!("  Method: {}", method.name);
    ///     }
    /// }
    /// # })
    /// ```
    ///
    /// # Structs
    ///
    /// * [`ServiceInfo`] - Represents information about a service, including its package name,
    ///   service name, and methods
    /// * [`MethodInfo`] - Represents information about a method, including its name.
    pub async fn list_services(&mut self) -> Result<Vec<ServiceInfo>, Box<dyn Error>> {
        let response = self
            .make_request(ServerReflectionRequest {
                host: "".to_string(),
                message_request: Some(MessageRequest::ListServices(String::new())),
            })
            .await?;

        if let MessageResponse::ListServicesResponse(services_response) = response {
            let mut services_info = Vec::new();

            for service in services_response.service {
                let descriptors = self.get_file_descriptor(service.name.clone()).await?;

                for file_descriptor in descriptors {
                    for service in file_descriptor.service {
                        let methods: Vec<MethodInfo> = service
                            .method
                            .into_iter()
                            .map(|method| {
                                let name = method.name.ok_or_else(|| {
                                    format!("Method name is missing for service {:?}", service.name)
                                })?;
                                let request = method.input_type.ok_or_else(|| {
                                    format!(
                                        "Request type is missing for method {:?} in service {:?}",
                                        name, service.name
                                    )
                                })?;
                                let response = method.output_type.ok_or_else(|| {
                                    format!(
                                        "Response type is missing for method {:?} in service {:?}",
                                        name, service.name
                                    )
                                })?;
                                Ok(MethodInfo {
                                    name,
                                    request,
                                    response,
                                })
                            })
                            .collect::<Result<Vec<MethodInfo>, Box<dyn Error>>>()?;

                        let package = file_descriptor.package.clone().ok_or_else(|| {
                            format!("Package name is missing for service {:?}", service.name)
                        })?;

                        let service_name = service.name.ok_or_else(|| {
                            format!("Service name is missing for package {}", package)
                        })?;

                        services_info.push(ServiceInfo {
                            package,
                            service: service_name,
                            methods,
                        });
                    }
                }
            }

            Ok(services_info)
        } else {
            Err("Expected a ListServicesResponse variant".into())
        }
    }

    /// Retrieves the file descriptors for the specified symbol from the server.
    ///
    /// This function sends a `ServerReflectionRequest` to the server to fetch the
    /// `FileDescriptorProto` for the provided symbol. It decodes the received file
    /// descriptors and returns them as a vector.
    ///
    /// # Arguments
    ///
    /// * `symbol` - A `String` containing the fully qualified name of the symbol
    ///   whose file descriptors are being requested.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<prost_types::FileDescriptorProto>, Box<dyn Error>>` - A result containing
    ///   a vector of `FileDescriptorProto` objects if the request is successful, or an error
    ///   if the request fails or the response is not of the expected type.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The request to the server fails.
    /// - The response from the server is not a `FileDescriptorResponse`.
    /// - The file descriptors cannot be decoded.
    ///
    /// # Example
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// let mut client = grpc_ease::reflection::ReflectionClient::new("http://localhost:50051".to_string()).await?;
    /// let descriptors = client.get_file_descriptor("my.package.MyService".to_string()).await?;
    /// for descriptor in descriptors {
    ///     println!("{:?}", descriptor);
    /// }
    /// # })
    /// ```
    pub async fn get_file_descriptor(
        &mut self,
        symbol: String,
    ) -> Result<Vec<prost_types::FileDescriptorProto>, Box<dyn Error>> {
        let response = self
            .make_request(ServerReflectionRequest {
                host: "".to_string(),
                message_request: Some(MessageRequest::FileContainingSymbol(symbol)),
            })
            .await?;

        if let MessageResponse::FileDescriptorResponse(descriptor_response) = response {
            let mut descriptors = Vec::new();
            for file_descriptor_proto in descriptor_response.file_descriptor_proto {
                let file_descriptor =
                    prost_types::FileDescriptorProto::decode(&file_descriptor_proto[..])?;
                descriptors.push(file_descriptor);
            }
            Ok(descriptors)
        } else {
            Err("Expected a FileDescriptorResponse variant".into())
        }
    }
}
