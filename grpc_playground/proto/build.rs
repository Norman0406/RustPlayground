fn main() {
    tonic_build::configure()
        .compile(&[
            "proto/hello/hello.proto",
            "proto/services/unary_service.proto",
            "proto/services/server_streaming_service.proto",
            "proto/services/client_streaming_service.proto",
            "proto/services/bidirectional_streaming_service.proto"],
            &["proto"])
        .expect("gRPC protobuf compilation failed")
}
