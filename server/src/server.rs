use tonic::{transport::Server, Request, Response, Status};
use helloworld::greeter_server::{Greeter, GreeterServer};
use helloworld::{HelloReply, HelloRequest};
use tonic_reflection::server::Builder as ReflectionBuilder;
use std::fs;

pub mod helloworld {
    tonic::include_proto!("helloworld");
}

#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = helloworld::HelloReply {
            message: format!("Hello {}!", request.into_inner().name).into(),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse().unwrap();
    let greeter = MyGreeter::default();

    println!("Server listening on {}", addr);

    // ディスクリプターファイルを実行時に読み込む
    let reflection_service = if let Ok(descriptor_bytes) = fs::read("proto/helloworld_descriptor.bin") {
        println!("Loading descriptor from proto/helloworld_descriptor.bin");
        ReflectionBuilder::configure()
            .register_encoded_file_descriptor_set(descriptor_bytes.as_ref())
            .build()
            .unwrap()
    } else {
        println!("WARNING: Descriptor file not found, running without reflection service");
        // フォールバック：リフレクションなしで実行
        ReflectionBuilder::configure()
            .build()
            .unwrap()
    };

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}