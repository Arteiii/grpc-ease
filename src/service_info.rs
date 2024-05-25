/// Represents information about an RPC method
#[derive(Debug)]
pub struct MethodInfo {
    /// The name of the RPC method
    pub name: String,
}

/// Represents information about a gRPC service, including its package name,
/// service name, and a list of RPC methods
#[derive(Debug)]
pub struct ServiceInfo {
    /// The package name of the gRPC service
    pub package: String,
    /// The name of the gRPC service
    pub service: String,
    /// A list of RPC methods available in the service
    pub methods: Vec<MethodInfo>,
}
