FROM ghcr.io/actions/runner:latest 

USER root

# 기본 유틸 설치
RUN apt-get update && apt-get install -y \
    ca-certificates curl xz-utils git && \
    rm -rf /var/lib/apt/lists/*

# BuildKit 설치
ARG BUILDKIT_VERSION=v0.26.3
RUN curl -L -o /tmp/buildkit.tar.gz https://github.com/moby/buildkit/releases/download/${BUILDKIT_VERSION}/buildkit-${BUILDKIT_VERSION}.linux-amd64.tar.gz && \
    tar -C /usr/local -xzf /tmp/buildkit.tar.gz && \
    rm /tmp/buildkit.tar.gz 

# Docker CLI 설치 (dockerd 없이 사용)
ARG DOCKER_CLI_VERSION=29.1.4
RUN curl -L -o /tmp/docker-cli.tar.gz https://download.docker.com/linux/static/stable/x86_64/docker-${DOCKER_CLI_VERSION}.tgz && \
    tar -C /usr/local/bin -xzf /tmp/docker-cli.tar.gz --strip-components=1 docker/docker && \
    rm /tmp/docker-cli.tar.gz

# Buildx 플러그인 설치
ARG BUILDX_VERSION=v0.11.2
RUN mkdir -p /usr/local/lib/docker/cli-plugins && \
    curl -L -o /usr/local/lib/docker/cli-plugins/docker-buildx https://github.com/docker/buildx/releases/download/${BUILDX_VERSION}/buildx-${BUILDX_VERSION}.linux-amd64 && \
    chmod +x /usr/local/lib/docker/cli-plugins/docker-buildx

# (선택) 사설 Harbor CA 인증서 번들 추가
# COPY harbor-ca.crt /usr/local/share/ca-certificates/harbor-ca.crt
# RUN update-ca-certificates

USER runner