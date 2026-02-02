#!/usr/bin/env bash
set -euo pipefail

REGISTRY="${REGISTRY:-harbor.yuiop.org/homepage}"
IMAGE_NAME="${IMAGE_NAME:-runner-buildkit}"
IMAGE_TAG="${IMAGE_TAG:-latest}"

# 로그인 (환경변수 )
echo "$HARBOR_PASSWORD" | docker login "$REGISTRY" -u "$HARBOR_USERNAME" --password-stdin

# 빌드 및 푸시
docker buildx build --platform linux/amd64,linux/arm64 -t "$REGISTRY/$IMAGE_NAME:$IMAGE_TAG" --push .