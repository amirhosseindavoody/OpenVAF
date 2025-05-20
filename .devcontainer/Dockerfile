FROM public.ecr.aws/ubuntu/ubuntu:22.04_stable

RUN apt-get update && \
    apt-get install -y \
        zsh \
        git \
        curl \
        wget \
        lsb-release \
        software-properties-common \
        gnupg \
        build-essential \
        binutils \
        zlib1g-dev \
        libzstd-dev

# Install llvm-16
RUN wget https://apt.llvm.org/llvm.sh && \
    chmod +x llvm.sh && \
    ./llvm.sh 16 all && \
    # Set llvm-16 and clang-16 as default
    update-alternatives --install /usr/bin/clang clang /usr/bin/clang-16 100 && \
    update-alternatives --install /usr/bin/clang++ clang++ /usr/bin/clang++-16 100 && \
    update-alternatives --install /usr/bin/llvm-config llvm-config /usr/bin/llvm-config-16 100 && \
    update-alternatives --install /usr/bin/llvm-ar llvm-ar /usr/bin/llvm-ar-16 100 && \
    update-alternatives --install /usr/bin/lld lld /usr/bin/lld-16 100 && \
    update-alternatives --install /usr/bin/clang-cl clang-cl /usr/bin/clang-cl-16 100 && \
    # Use clang as default compiler
    update-alternatives --install /usr/bin/cc cc /usr/bin/clang 100 && \
    update-alternatives --install /usr/bin/llvm-lib llvm-lib /usr/bin/llvm-ar-16 100
    
# Install rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.65.0

RUN add-apt-repository ppa:deadsnakes/ppa && \
    apt-get update && \
    apt-get install -y \
        python3.10 \
        python3.10-dev \
        python3.10-venv \
        python3.10-distutils && \
    update-alternatives --install /usr/bin/python3 python3 /usr/bin/python3.10 1 && \
    update-alternatives --install /usr/bin/python python /usr/bin/python3.10 1

# # Clean up
# RUN rm -rf /var/lib/apt/lists/* && \
#     apt-get clean && \
#     apt-get autoremove

ENV PATH=/root/.cargo/bin:$PATH
ENV CARGO_HOME=/root/.cargo
ENV RUSTUP_HOME=/root/.rustup
ENV CC=clang
ENV CXX=clang++
ENV AR=llvm-ar
ENV PYO3_PYTHON=/usr/bin/python