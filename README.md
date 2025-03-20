# **Rust Microservice with Envoy**

This repository provides a foundation for building gRPC microservices in Rust combined with an Envoy proxy.

## **Project Purpose**

The goal of this project is to establish a foundational distributed architecture using Rust. Rust was chosen primarily for its strong type safety and "Write once, run anywhere" characteristics. These features enable the development of high-performance and secure microservices.

The current architecture consists of the following components:
- **gRPC server implemented in Rust**: A high-performance backend service.
- **Envoy proxy**: Acts as an API gateway and supports the gRPC-Web protocol.
- **Docker**: Containerizes all components to provide a reproducible environment.

## **How to Start the Application**

### **Prerequisites**
- Docker and Docker Compose must be installed.
- A gRPC client (e.g., `grpcurl`) is useful for testing.

### **Startup Steps**

1. Clone the repository:
```bash
git clone https://github.com/kazuma0606/rust-microservice-with-envoy.git
cd rust-microservice-with-envoy
```

2. Build and start the Docker containers:
```bash
docker-compose build
docker-compose up
```

3. Test the service:
```bash
# Using grpcurl
grpcurl -plaintext -d '{"name": "Rust"}' localhost:8080 helloworld.Greeter/SayHello

# Expected output
{
  "message": "Hello Rust!"
}
```

## **Future Plans**

This project is planned to evolve in the following directions:

1. **Building a local Kubernetes environment**: Enhancing microservice orchestration and management.
2. **Expanding the microservices ecosystem**: Implementing additional services such as database integration.
3. **Diverse application deployment**:
   - Initially developed as a desktop application.
   - Once stable, transitioning to a web application via an intermediary server.

The long-term goal is to create a "native cloud" environment that replicates cloud services such as AWS, GCP, and Azure locally. This setup will allow unrestricted **try & error** development in an environment equivalent to production.

## **Contributions**

Contributions in any form, including bug reports, feature requests, and pull requests, are highly welcome.

## **License**

This project is released under the MIT License.
