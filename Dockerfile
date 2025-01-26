FROM debian:bookworm

# Update default packages
RUN apt-get update -y && apt-get upgrade -y

# Install tools
RUN apt-get install -y linux-perf wget curl build-essential

# Install rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Set path
ENV PATH="/root/.cargo/bin:/usr/lib/linux-tools-6.8.0-51:${PATH}"

# Install flamegraph
RUN cargo install flamegraph
