# development stage
FROM rust:slim AS development 
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y \
    git \
    curl \
    wget \
    unzip \
    build-essential \
    pkg-config \
    libssl-dev \
    ca-certificates \
    git \
    zsh \
    sudo \
    redis-tools \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Rust 환경 설정
RUN rustup default stable \
    && rustup update stable \
    && rustup component add rust-src \
    && rustup component add clippy \
    && rustup component add rustfmt
# Install common Rust development tools
# RUN cargo install \
#     cargo-watch \
#     cargo-edit \
#     cargo-audit \
#     cargo-outdated



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

RUN groupadd --gid 1000 vscode \
    && useradd --uid 1000 --gid vscode --shell /bin/zsh --create-home vscode \
    && echo vscode ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/vscode \
    && chmod 0440 /etc/sudoers.d/vscode
USER vscode
WORKDIR /workspace
RUN chown -R vscode:vscode /workspace

# ohymyzsh 설치 및 플러그인 설정
RUN sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)" \
    && git clone https://github.com/zsh-users/zsh-autosuggestions ${ZSH_CUSTOM:-~/.oh-my-zsh/custom}/plugins/zsh-autosuggestions \
    && git clone https://github.com/zsh-users/zsh-syntax-highlighting.git ${ZSH_CUSTOM:-~/.oh-my-zsh/custom}/plugins/zsh-syntax-highlighting
COPY --chown=vscode:vscode .zshrc /home/vscode/.zshrc
RUN echo 'source /etc/profile.d/oracle.sh' >> /home/vscode/.zshrc

# nvm 및 Node.js 설치
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.3/install.sh | bash \
    && export NVM_DIR="$HOME/.nvm" \
    && [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh" \
    && nvm install --lts 

CMD ["/bin/zsh"]