BUILD_DIR=build
DOCKER_IMAGE ?= owl:devel
RUNS ?= 2

all: analyzer monitor

.PHONY: analyzer monitor prepare clean docker-build test-docker

monitor: prepare
	make -C src/owl_monitor
	mkdir -p ${BUILD_DIR}/lib 
	cp src/owl_monitor/gpu_trace/gpu_trace.so ${BUILD_DIR}/lib/gpu_trace.so 
	cp src/owl_monitor/cpu_trace/obj-intel64/cpu_trace.so ${BUILD_DIR}/lib/cpu_trace.so

prepare:
	mkdir -p ${BUILD_DIR}

analyzer:
	cd src/owl_analyzer/ && cargo build --release

clean:
	rm -f ${BUILD_DIR}/lib/gpu_trace.so ${BUILD_DIR}/lib/cpu_trace.so 
	cd src/owl_monitor && make clean 

# Build a GPU-enabled Docker image suitable for bind-mount workflows
.PHONY: docker-build
docker-build:
	docker build -t $(DOCKER_IMAGE) .

# One-shot: build analyzer/monitor/example and run the analyzer inside Docker
# Usage:
#   make ARCH=86 test-docker         # default RUNS=2
#   make ARCH=86 RUNS=3 test-docker  # customize runs per phase
.PHONY: test-docker
test-docker: docker-build
	docker run --gpus all --rm -t \
	  -v "$$PWD":/root/owl \
	  -w /root/owl \
	  $(DOCKER_IMAGE) \
	  bash -lc 'make clean || true && make ARCH=$(ARCH) && cd example/cuda-examples && make clean && make && \
	    cd /root/owl && \
	    src/owl_analyzer/target/release/owl_analyzer \
	      --cmds-file example/cuda-examples/cmds \
	      --rand-cmd "/root/owl/src/owl-wrapper /root/owl/example/cuda-examples/randaccess" \
	      -t $(RUNS)'
