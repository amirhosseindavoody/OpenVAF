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
        binutils && \
    mkdir -p /custom_home && \
    export HOME=/custom_home && \
    mkdir ${HOME}/workspace && \
    
    # Install and config oh-my-zsh
    git clone https://github.com/amirhosseindavoody/dotfiles.git ${HOME}/dotfiles && \
    cd ${HOME}/dotfiles && \
    ./setup.sh config.yaml && \
    
    # Change the default shell to zsh for all users in /etc/passwd
    sed -i 's|/bin/bash$|/usr/bin/zsh|' /etc/passwd && \
    
    # Install llvm-16
    cd ${HOME} && \
    wget https://apt.llvm.org/llvm.sh && \
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
    update-alternatives --install /usr/bin/llvm-lib llvm-lib /usr/bin/llvm-ar-16 100 && \
    
    # Install zstd lib
    apt-get install -y \
        zlib1g-dev \
        libzstd-dev && \
    
    # Install rust
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --no-modify-path -y --default-toolchain none && \
    ${HOME}/.cargo/bin/rustup install 1.65.0 && \
    ${HOME}/.cargo/bin/rustup default 1.65.0 && \
    
    # Clean up
    rm -rf /var/lib/apt/lists/* && \
    apt-get clean && \
    apt-get autoremove

ENV HOME=/custom_home
ENV PATH=$HOME/.cargo/bin:$PATH
ENV CARGO_HOME=$HOME/.cargo
ENV RUSTUP_HOME=$HOME/.rustup
ENV CC=clang
ENV CXX=clang++
ENV AR=llvm-ar