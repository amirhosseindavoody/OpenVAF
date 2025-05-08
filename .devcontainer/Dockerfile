FROM ubuntu:latest

RUN apt-get update \
    && \
    apt-get install -y \
    git \
    clang \
    clangd \
    autoconf \
    automake \
    bison \
    flex \
    g++ \
    libx11-dev \
    libxaw7-dev \
    libtool \
    libreadline-dev \
    libomp-dev \
    make \
    cmake \
    ninja-build \
    && \
    rm -rf /var/lib/apt/lists/* && \
    apt-get clean && \
    apt-get autoremove
