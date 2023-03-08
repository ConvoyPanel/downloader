FROM rust:1.67

RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev

WORKDIR /workspace