use std::io::Result;

fn main() -> Result<()> {
    tonic_prost_build::configure()
        .build_server(false)
        .compile_protos(
            &["protos/api.proto"],   // Update the path to the .proto files 
            &["protos"],             // Update the search path for proto files
        )?;
    Ok(())
}