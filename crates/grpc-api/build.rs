fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/generated")
        .compile_protos(
            &["proto/cvad.proto"], // Update the path to the .proto files
            &["proto"],            // Update the search path for proto files
        )
        .expect("Failed to compile protos");
    Ok(())
}
