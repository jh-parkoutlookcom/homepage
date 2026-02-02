# build stage for react
FROM node:latest AS react-builder
WORKDIR /app
COPY frontend/ .
RUN npm install && npm run build

# build stage for rust
FROM rust:latest AS rust-builder
RUN apt-get update && apt-get install -y protobuf-compiler
WORKDIR /app
COPY . .
RUN cargo build --release -p web-server

# production stage
FROM debian:stable-slim AS production
RUN apt-get update && apt-get install -y wget unzip libssl-dev

# oracle client installation
RUN mkdir -p /opt/oracle \
    && cd /opt/oracle \
    && wget -q https://download.oracle.com/otn_software/linux/instantclient/2390000/instantclient-basic-linux.x64-23.9.0.25.07.zip \
    && wget -q https://download.oracle.com/otn_software/linux/instantclient/2390000/instantclient-sqlplus-linux.x64-23.9.0.25.07.zip \
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

COPY --from=rust-builder /app/target/release/web-server /app/web-server
COPY --from=react-builder /app/dist /frontend
EXPOSE 8080 30015
CMD ["/app/web-server"]
