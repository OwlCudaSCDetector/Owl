FROM nvidia/cuda:11.6.1-devel-ubuntu20.04

# Use bash and ensure cargo is on PATH for non-interactive shells
SHELL ["/bin/bash", "-lc"]

WORKDIR /root

RUN apt update && \
        DEBIAN_FRONTEND=noninteractive apt install -y --no-install-recommends \
            build-essential ninja-build python3 python3-pip python3-setuptools \
            curl git g++-multilib wget bc ca-certificates clang \
            pkg-config libfontconfig1-dev && \
        rm -rf /var/lib/apt/lists/* && \
        python3 -m pip install --no-cache-dir pyyaml typing-extensions numpy scipy matplotlib

# Install CMake 3.20
RUN wget -q https://cmake.org/files/v3.20/cmake-3.20.0-linux-x86_64.tar.gz -O /tmp/cmake.tar.gz &&  \
        tar -zxf /tmp/cmake.tar.gz -C /usr --strip-components=1 && \
        rm -f /tmp/cmake.tar.gz

# Install Rust (rustup) and expose cargo in PATH
RUN curl --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/rustup.rs && \
        sh /tmp/rustup.rs -y && \
        echo 'export PATH="$HOME/.cargo/bin:$PATH"' > /etc/profile.d/cargo.sh
ENV PATH=/root/.cargo/bin:${PATH}

# Optional: clone and build inside image (disabled by default)
ARG CLONE=0
ARG REPO_URL=https://github.com/OwlCudaSCDetector/Owl.git
RUN if [[ "$CLONE" == "1" ]]; then \
        git clone --depth 1 "$REPO_URL" /root/owl && \
        cd /root/owl && make ARCH=86; \
    fi

# Default working directory when running with a bind mount
WORKDIR /root/owl