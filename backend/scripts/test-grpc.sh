#!/bin/bash 
# filepath: /workspace/scripts/test-grpc.sh

set -e

echo "Testing gRPC service..."

# Check if grpcurl is installed
if ! command -v grpcurl & > /dev/null; then
    echo "Installing grpccurl..."
    GRPCURL_VERSION=$(curl -s https://api.github.com/repos/fullstorydev/grpcurl/releases/latest | grep '"tag_name"' | sed -E 's/.*"v([^"]+)".*/\1/')
    curl -L "https://github.com/fullstorydev/grpcurl/releases/download/v${GRPCURL_VERSION}/grpcurl_${GRPCURL_VERSION}_linux_x86_64.tar.gz" | tar -xz
    sudo mv grpcurl /usr/local/bin
fi

# Test gRPC endpoint
grpcurl -plaintext \
    -import-path ./creates/grpc-api/proto \
    -proto cvad.proto \
    -d '{"cvad_url": "https://nuc-citrix.yuiop.org", "site_name": "nuc-citrix"}' \
    localhost:50051 \
    cvad.v1.Cvad/GetExpireDate 

echo "gRPC test completed!"
