# Prefer GCC 10 as nvcc host compiler when available (CUDA 11.x compatibility)
CXX?=g++
ifneq ($(shell which g++-10 2>/dev/null),)
	CXX:=g++-10
endif

NVCC=nvcc -ccbin=$(CXX) -D_FORCE_INLINES

NVCC_VER_REQ=10.1
NVCC_VER=$(shell $(NVCC) --version | grep release | cut -f2 -d, | cut -f3 -d' ')
NVCC_VER_CHECK=$(shell echo "${NVCC_VER} >= $(NVCC_VER_REQ)" | bc)

ifeq ($(NVCC_VER_CHECK),0)
$(error ERROR: nvcc version >= $(NVCC_VER_REQ) required to compile an nvbit tool! Instrumented applications can still use lower versions of nvcc.)
endif

NVBIT_PATH=../core
INCLUDES=-I$(NVBIT_PATH)

LIBS=-L$(NVBIT_PATH) -lnvbit
NVCC_PATH=-L $(subst bin/nvcc,lib64,$(shell which nvcc | tr -s /))

SOURCES=$(wildcard *.cu) helper/pipe/pipe.cu

OBJECTS=$(SOURCES:.cu=.o) cfg.o  dump.o bt.o#./dcfg/dcfg.pb.o
ARCH?=75

mkfile_path := $(abspath $(lastword $(MAKEFILE_LIST)))
current_dir := $(notdir $(patsubst %/,%,$(dir $(mkfile_path))))

NVBIT_TOOL=$(current_dir).so

all: $(NVBIT_TOOL)

$(NVBIT_TOOL): $(OBJECTS) $(NVBIT_PATH)/libnvbit.a
	$(NVCC) -arch=sm_$(ARCH) -O3 $(OBJECTS) $(LIBS) $(NVCC_PATH) -lcuda -lcudart_static -shared -o $@

%.o: %.cu $(SOURCES)
	$(NVCC) -dc -c -std=c++11 $(INCLUDES) -Xptxas -cloning=no -Xcompiler -Wall -arch=sm_$(ARCH) -O3 -Xcompiler -fPIC $< -o $@ 


cfg.o: cfg.cpp
	clang++ -c -fPIC $< -o $@ -Wnon-c-typedef-for-linkage

dump.o: dump.cpp
	clang++ -c -fPIC $< -o $@

bt.o: bt.cpp
	clang++ -c -fPIC $< -o $@ -Wnon-c-typedef-for-linkage

device_trace.o: device_trace.cu
	$(NVCC) $(INCLUDES) -maxrregcount=24 -Xptxas -astoolspatch --keep-device-functions -arch=sm_$(ARCH) -Xcompiler -Wall -Xcompiler -fPIC -c $< -o $@

clean:
	rm -f *.so *.o core helper/pipe/pipe.o
