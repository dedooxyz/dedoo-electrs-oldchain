FROM debian:latest

# Install dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    git \
    libclang-dev

# Install Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y --default-toolchain 1.75.0
RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc
RUN bash -c 'source ~/.bashrc; rustup install stable'

# Clone and build electrs
WORKDIR /esplora/electrs
COPY . .
RUN bash -c 'source ~/.bashrc; cargo build'

# Set up directories
RUN mkdir -p /junkcoin/.junkcoin/.electrs
RUN bash -c 'ulimit -n $(ulimit -n -H)'

# Run electrs
COPY docker-entrypoint.sh /docker-entrypoint.sh
CMD ["bash", "/docker-entrypoint.sh"]