fn main() {
    tonic_build::configure()
        .compile(
            &[
                "proto/chat/message.proto",
                "proto/chat/service.proto",
                "proto/chat/user.proto",
            ],
            &["proto"],
        )
        .expect("gRPC protobuf compilation failed")
}
