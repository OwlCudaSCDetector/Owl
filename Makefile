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

.PHONY:
clean:
	rm -f gpu_trace.so
	cd src/owl_monitor && make clean 
