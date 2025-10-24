all: gpu_trace cpu_trace

gpu_trace: cpu_trace
	+cd gpu_trace && $(MAKE) ARCH=$(ARCH)

cpu_trace: check_and_download_pin
	+$(MAKE) -C cpu_trace/

.PHONY: check_and_download_pin
check_and_download_pin:
	if [ ! -d "./pin_root" ]; then \
		echo "download Intel Pin..."; \
		wget https://software.intel.com/sites/landingpage/pintool/downloads/pin-3.28-98749-g6643ecee5-gcc-linux.tar.gz; \
		tar -xzvf pin-3.28-98749-g6643ecee5-gcc-linux.tar.gz; \
		mv pin-3.28-98749-g6643ecee5-gcc-linux  pin_root; \
		rm pin-3.28-98749-g6643ecee5-gcc-linux.tar.gz; \
	fi

clean:
	cd gpu_trace && $(MAKE) clean
	cd cpu_trace && $(MAKE) clean
