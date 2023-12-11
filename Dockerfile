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

# configure ssh key for git
RUN git clone https://siriuswhiter:3a0d6389eab51d867a64c10d94b7bb8d@gitee.com/CyoeeA9e/owls.git

# for me
RUN apt install -y vim

RUN apt install -y zsh && \
        sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)" && \
        echo "export LD_LIBRARY_PATH=/usr/local/lib\nexport PATH=/usr/local/cuda/bin:$PATH\nDISABLE_UNTRACKED_FILES_DIRTY=\"true\"" >> .zshrc && \
        chsh --shell /bin/zsh root

RUN apt install -y openssh-server && \
        service ssh start
