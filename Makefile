BUILD_TYPE=release
BUILD_DIR=build
INSTALL_PREFIX=/usr/local
NINJA=1
FLAGS=-j 64
CFLAGS=$(FLAGS)
CXXFLAGS=$(FLAGS)


all: llvm torch monitor

monitor: llvm_prepare
	make -C src/owl_monitor
	mkdir -p ${BUILD_DIR}/lib 
	cp src/owl_monitor/gpu_trace/gpu_trace.so ${BUILD_DIR}/lib/gpu_trace.so 
	cp src/owl_monitor/cpu_trace/obj-intel64/cpu_trace.so ${BUILD_DIR}/lib/cpu_trace.so

profiler:
	make -C experiment/owl_checker/profiler

.PHONY:
clean:
	rm -f gpu_trace.so
	cd src/owl_monitor && make clean 

llvm_prepare:
	mkdir -p ${BUILD_DIR}

llvm_configure: llvm_prepare
	cmake -S dependency/llvm-project/llvm -B ${BUILD_DIR} \
	-DCMAKE_BUILD_TYPE=$(if $(DEBUG),Debug,Release) \
	-DLLVM_TARGETS_TO_BUILD="X86;NVPTX" \
	-DCMAKE_EXPORT_COMPILE_COMMANDS=1 \
	-DBUILD_SHARED_LIBS=ON \
	-DLLVM_TARGET_ARCH=host \
	-DLLVM_ENABLE_ASSERTIONS=ON \
	-DINSTALL_PREFIX_PATH=${INSTALL_PREFIX} \
	-DLLVM_ENABLE_PROJECTS="lld;clang;openmp" \
	-G $(if $(NINJA),"Ninja","Unix Makefiles")

llvm: llvm_configure
	cmake --build ${BUILD_DIR} --target install

analyzer:
	cd src/owl_analyzer/ && cargo build --release