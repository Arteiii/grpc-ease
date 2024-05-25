use std::error::Error;
use prost::Message;
use tokio_stream::StreamExt;
use tonic::{Request, transport::Endpoint};
use tonic::transport::Channel;
use tonic_reflection::pb::server_reflection_client::ServerReflectionClient;
use tonic_reflection::pb::{ServerReflectionRequest};
use tonic_reflection::pb::server_reflection_request::MessageRequest;
use tonic_reflection::pb::server_reflection_response::MessageResponse;
use crate::service_info::{ServiceInfo, MethodInfo};

pub struct ReflectionClient {
    client: ServerReflectionClient<tonic::transport::Channel>,
}

impl ReflectionClient {
    pub async fn new(endpoint: String) -> Result<Self, Box<dyn Error>> {
        let channel = Channel::from_shared(endpoint)?.connect().await?;
        Ok(Self {
            client: ServerReflectionClient::new(channel),
        })
    }

    pub async fn connect(addr: &str) -> Result<Self, Box<dyn Error>> {
        let endpoint = Endpoint::new(addr.to_string())?.connect().await?;
        Ok(Self {
            client: ServerReflectionClient::new(endpoint),
        })
    }

    async fn make_request(&mut self, request: ServerReflectionRequest) -> Result<MessageResponse, Box<dyn Error>> {
        let request = Request::new(tokio_stream::once(request));
        let mut inbound = self.client.server_reflection_info(request).await?.into_inner();

        if let Some(response) = inbound.next().await {
            return Ok(response?.message_response.expect("some MessageResponse"));
        }

        Err("No response received".into())
    }

    pub async fn list_services(&mut self) -> Result<Vec<ServiceInfo>, Box<dyn Error>> {
        let response = self.make_request(ServerReflectionRequest {
            host: "".to_string(),
            message_request: Some(MessageRequest::ListServices(String::new())),
        }).await?;

        if let MessageResponse::ListServicesResponse(services_response) = response {
            let mut services_info = Vec::new();

            for service in services_response.service {
                let descriptors = self.get_file_descriptor(service.name.clone()).await?;

                for file_descriptor in descriptors {
                    for service in file_descriptor.service {
                        let methods: Vec<MethodInfo> = service.method.into_iter()
                            .map(|method| {
                                method.name.ok_or_else(|| {
                                    format!("Method name is missing for service {:?}", service.name)
                                }).map(|name| MethodInfo { name })
                            })
                            .collect::<Result<Vec<MethodInfo>, _>>()?;

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


    async fn get_file_descriptor(&mut self, symbol: String) -> Result<Vec<prost_types::FileDescriptorProto>, Box<dyn Error>> {
        let response = self.make_request(ServerReflectionRequest {
            host: "".to_string(),
            message_request: Some(MessageRequest::FileContainingSymbol(symbol)),
        }).await?;

        if let MessageResponse::FileDescriptorResponse(descriptor_response) = response {
            let mut descriptors = Vec::new();
            for file_descriptor_proto in descriptor_response.file_descriptor_proto {
                let file_descriptor = prost_types::FileDescriptorProto::decode(&file_descriptor_proto[..])?;
                descriptors.push(file_descriptor);
            }
            Ok(descriptors)
        } else {
            Err("Expected a FileDescriptorResponse variant".into())
        }
    }
}
