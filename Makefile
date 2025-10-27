BUILD_DIR=build
REPO:=$(abspath .)
RUNS?=2

all: analyzer monitor

monitor: prepare
	make -C src/owl_monitor
	mkdir -p ${BUILD_DIR}/lib 
	cp src/owl_monitor/gpu_trace/gpu_trace.so ${BUILD_DIR}/lib/gpu_trace.so 
	cp src/owl_monitor/cpu_trace/obj-intel64/cpu_trace.so ${BUILD_DIR}/lib/cpu_trace.so

prepare:
	mkdir -p ${BUILD_DIR}

analyzer:
	cd src/owl_analyzer/ && cargo build --release

.PHONY: test-native
test-native: analyzer monitor
	@echo "[test-native] Building example/cuda-examples"
	$(MAKE) -C example/cuda-examples
	@echo "[test-native] Ensuring cmds.local exists with native paths"
	@if [ ! -f example/cuda-examples/cmds.local ]; then \
		echo "$(REPO)/src/owl-wrapper $(REPO)/example/cuda-examples/randaccess" > example/cuda-examples/cmds.local; \
	fi
	@echo "[test-native] Running analyzer (runs=$(RUNS))"
	$(REPO)/src/owl_analyzer/target/release/owl_analyzer \
	  --cmds-file $(REPO)/example/cuda-examples/cmds.local \
	  --rand-cmd "$(REPO)/src/owl-wrapper $(REPO)/example/cuda-examples/randaccess" \
	  -t $(RUNS)

.PHONY:
clean:
	rm -f ${BUILD_DIR}/lib/gpu_trace.so ${BUILD_DIR}/lib/cpu_trace.so 
	cd src/owl_monitor && make clean 
