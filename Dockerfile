FROM nvidia/cuda:11.6.1-devel-ubuntu20.04

WORKDIR /root

RUN apt update && \
        apt install -y build-essential ninja-build python3   python3-pip python3-setuptools curl git g++-multilib wget bc && \
        python3 -m pip install pyyaml typing-extensions numpy scipy  matplotlib

#install cmake-3.20
RUN wget https://cmake.org/files/v3.20/cmake-3.20.0-linux-x86_64.tar.gz -O /tmp/cmake.tar.gz &&  \
        tar -zxvf /tmp/cmake.tar.gz -C /usr --strip-components=1

# install rust
RUN curl --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/rustup.rs && \
        sh /tmp/rustup.rs -y && echo "source \"$HOME/.cargo/env\"\n" >> .zshrc

# clone code
RUN git clone git@github.com:OwlCudaSCDetector/Owl.git owl

RUN cd owl && make