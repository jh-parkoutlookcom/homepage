# developement stage
FROM rust:1-slim-trixie AS developement

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get install -y sudo zsh build-essential libssl-dev pkg-config libpq-dev libaio1t64 wget unzip git curl \
    && rm -rf /var/lib/apt/lists/*

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

# Redis tools 설치 (개발용)
RUN apt-get update && apt-get install -y redis-tools \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*


RUN groupadd --gid 1000 vscode \
    && useradd --uid 1000 --gid vscode --shell /bin/zsh --create-home vscode \
    && echo vscode ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/vscode \
    && chmod 0440 /etc/sudoers.d/vscode

WORKDIR /backend 
RUN chown -R vscode:vscode /backend

# 환경변수 설정
ENV ORACLE_HOME=/opt/oracle/instantclient_23_9
ENV LD_LIBRARY_PATH=$LD_LIBRARY_PATH:$ORACLE_HOME
ENV PATH=$PATH:$ORACLE_HOME
ENV RUST_LOG=debug
ENV ENVIRONMENT=development

USER vscode

# 셸 환경 설정
RUN sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"
RUN git clone https://github.com/zsh-users/zsh-autosuggestions ${ZSH_CUSTOM:-~/.oh-my-zsh/custom}/plugins/zsh-autosuggestions
RUN git clone https://github.com/zsh-users/zsh-syntax-highlighting.git ${ZSH_CUSTOM:-~/.oh-my-zsh/custom}/plugins/zsh-syntax-highlighting
COPY --chown=vscode:vscode .zshrc /home/vscode/.zshrc
RUN echo 'source /usr/local/cargo/env' >> /home/vscode/.zshrc \
    && echo 'source /etc/profile.d/oracle.sh' >> /home/vscode/.zshrc
CMD ["/bin/zsh"]

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