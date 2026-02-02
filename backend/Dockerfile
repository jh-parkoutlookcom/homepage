# Production Dockerfile
FROM rust:slim AS builder 
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    ca-certificates \
    redis-tools \
    protobuf-compiler \
    libprotobuf-dev \
    wget \
    unzip \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*


# Rust 환경 설정
RUN rustup default stable \
    && rustup update stable \
    && rustup component add rust-src \
    && rustup component add clippy \
    && rustup component add rustfmt

# oracle client installation
RUN mkdir -p /opt/oracle \
    && cd /opt/oracle \
    && wget -q http://truenas.yuiop.org:30035/oracle/instantclient-basic-linux.x64-23.9.0.25.07.zip \
    && wget -q http://truenas.yuiop.org:30035/oracle/instantclient-sqlplus-linux.x64-23.9.0.25.07.zip \
    && unzip -q instantclient-basic-linux.x64-23.9.0.25.07.zip \
    && unzip -qo instantclient-sqlplus-linux.x64-23.9.0.25.07.zip \
    && rm -f *.zip \
    && echo /opt/oracle/instantclient_23_9 > /etc/ld.so.conf.d/oracle-instantclient.conf \
    && ldconfig

# Oracle 환경변수를 시스템 전체에 적용
RUN echo '#!/bin/sh' > /etc/profile.d/oracle.sh \
    && echo 'export ORACLE_HOME=/opt/oracle/instantclient_23_9' >> /etc/profile.d/oracle.sh \
    && echo 'export LD_LIBRARY_PATH=$ORACLE_HOME:$LD_LIBRARY_PATH' >> /etc/profile.d/oracle.sh \
    && echo 'export PATH=$ORACLE_HOME:$PATH' >> /etc/profile.d/oracle.sh \
    && chmod +x /etc/profile.d/oracle.sh

WORKDIR /workspace 
COPY . .
RUN cargo build --release -p web-server  

# Production stage
FROM debian:stable-slim AS production
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    redis-tools \
    wget \
    unzip \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# oracle client installation
RUN mkdir -p /opt/oracle \
    && cd /opt/oracle \
    && wget -q http://truenas.yuiop.org:30035/oracle/instantclient-basic-linux.x64-23.9.0.25.07.zip \
    && wget -q http://truenas.yuiop.org:30035/oracle/instantclient-sqlplus-linux.x64-23.9.0.25.07.zip \
    && unzip -q instantclient-basic-linux.x64-23.9.0.25.07.zip \
    && unzip -qo instantclient-sqlplus-linux.x64-23.9.0.25.07.zip \
    && rm -f *.zip \
    && echo /opt/oracle/instantclient_23_9 > /etc/ld.so.conf.d/oracle-instantclient.conf \
    && ldconfig

# Oracle 환경변수를 시스템 전체에 적용
RUN echo '#!/bin/sh' > /etc/profile.d/oracle.sh \
    && echo 'export ORACLE_HOME=/opt/oracle/instantclient_23_9' >> /etc/profile.d/oracle.sh \
    && echo 'export LD_LIBRARY_PATH=$ORACLE_HOME:$LD_LIBRARY_PATH' >> /etc/profile.d/oracle.sh \
    && echo 'export PATH=$ORACLE_HOME:$PATH' >> /etc/profile.d/oracle.sh \
    && chmod +x /etc/profile.d/oracle.sh

WORKDIR /app
COPY --from=builder /workspace/target/release/web-server /app/web-server
EXPOSE 8080 50051
CMD ["/app/web-server"]