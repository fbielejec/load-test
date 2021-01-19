fn main() {
    tonic_build::compile_protos("proto/pingpong.proto").unwrap();
    tonic_build::compile_protos("proto/chat.proto").unwrap();
}
