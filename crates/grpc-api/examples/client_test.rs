use grpc_api::generated::cvad::v1::{GetExpireDateRequest, cvad_client::CvadClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the gRPC server
    let mut client = CvadClient::connect("http://localhost:50051").await?;
    println!("Connected to gRPC server");

    // Make the request
    let request = tonic::Request::new(GetExpireDateRequest {
        cvad_url: "https://nuc-citrix.yuiop.org".to_string(),
        site_name: "nuc-citrix".to_string(),
    });
    println!("Sending request...");

    let response = client.get_expire_date(request).await?;
    let expire_info = response.into_inner();

    println!("Found: {}", expire_info.found);
    println!("Expire Date: {}", expire_info.expire_rfc3339);
    if !expire_info.error.is_empty() {
        println!("Error: {}", expire_info.error);
    }

    Ok(())
}
