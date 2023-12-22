CC = nvcc
CFLAGS = --generate-line-info -gencode arch=compute_86,code=sm_86

all: randaccess


randaccess: randaccess.cu 
	${CC} randaccess.cu ${CFLAGS} -o randaccess 

.PHONY: clean
clean:
	-rm -f ${target} *.ll core *.o *.s

test:
	@cd .. && make
	@../../src/owl_analyzer/target/release/owl_analyzer --cmds-file ./cmds --rand-cmd "/root/owl/src/owl-wrapper /root/owl/example/cuda-examples/randaccess" -t 2

