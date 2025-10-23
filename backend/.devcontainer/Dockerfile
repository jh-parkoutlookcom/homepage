# developement stage
FROM harbor.yuiop.org/homepage/backend-devenv:v0.3 AS developement

USER vscode
WORKDIR /backend 
# RUN chown -R vscode:vscode /backend


# build stage
FROM developement AS build
COPY --chown=vscode:vscode . .
RUN cargo build --release
RUN cargo install --path .


# production stage
FROM debian:trixie-slim AS production
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=build /usr/local/cargo/bin/backend /usr/local/bin/backend
EXPOSE 8080
CMD ["backend"]