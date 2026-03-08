pub mod dto;
pub mod grpc_handler;

pub mod proto {
    pub mod aggregator {
        tonic::include_proto!("authpulse.v1.aggregator");
    }
}
