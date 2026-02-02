#!/bin/sh 
set -e 
# Build script for the project
echo "Starting build process..."
# Build Docker image
docker build --target production -t homepage-backend: .
echo "Build process completed."