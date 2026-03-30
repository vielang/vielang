/// Build script for vl-cluster — compiles edge gRPC proto.
///
/// Uses `protox` (pure Rust proto compiler) — no need to install `protoc`.
/// tonic-build 0.14 moved prost/proto compilation to `tonic-prost-build`.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fds = protox::compile(["proto/edge.proto"], ["proto"])?;

    tonic_prost_build::configure()
        .build_server(true)
        .build_client(false)    // Edge client is the external TB Edge Java process
        .compile_fds(fds)?;

    Ok(())
}
