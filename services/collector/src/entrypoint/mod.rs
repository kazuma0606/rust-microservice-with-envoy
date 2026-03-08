pub mod dto;
pub mod grpc_handler;

pub mod proto {
    pub mod collector {
        tonic::include_proto!("authpulse.v1.collector");
    }
}
