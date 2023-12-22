GPUCRYPTO_DIR = ./

include Makefile.in


LIBDIR = $(OPENSSL_LIBDIR) \
	 $(CUDA_LIBDIR) \
	 $(GPUCRYPTO_LIBDIR)

LIBS =  -lpthread $(CUDA_LIBS) $(OPENSSL_LIB) $(GPYCRYPTO_LIB)
OBJS_DIR = objs
TARGET_DIR = lib
TARGET_FILE = libgpucrypto.a
TARGET = $(addprefix $(TARGET_DIR)/, $(TARGET_FILE))

.SUFFIXES : .cu .cc .o

CU_SRC_FILES = $(wildcard *.cu)
CC_SRC_FILES = $(wildcard *.cc)
HEADER_FILES = $(wildcard *.hh) $(wildcard *.h)

SRC_FILES = $(CU_SRC_FILES) $(CC_SRC_FILES)
OBJS_FILE = $(CU_SRC_FILES:.cu=.o) $(CC_SRC_FILES:.cc=.o)

OBJS = $(addprefix $(OBJS_DIR)/, $(OBJS_FILE))
DEPS = Makefile.dep

OBJ_DIR := objs
$(shell mkdir -p $(OBJ_DIR))

all: $(TARGET) test

$(TARGET): $(DEPS) $(OBJS) | $(TARGET_DIR) $(OBJS_DIR)
	ar rcs $@ $(OBJS)

$(TARGET_DIR):
	mkdir $(TARGET_DIR)

$(OBJS_DIR):
	mkdir $(OBJS_DIR)

$(DEPS): $(SRC_FILES) $(HEADER_FILES)
	$(CC) -MM -MP -x c++ $(CU_SRC_FILES) $(CC_SRC_FILES) | sed 's![^:]*.o:!objs/&!g' > Makefile.dep

$(OBJS_DIR)/%.o : %.cc
	$(CC) $(CFLAGS) $(INC) $(NVINC) -c $< -o $@

$(OBJS_DIR)/%.o : %.cu
	$(NVCC) $(NVCFLAGS) $(NVINC) -c $< -o $@

.PHONY : test clean doc

test: $(TARGET)
	make -C ./test

test_aes: test
	/root/owl/src/owl_analyzer/target/release/owl_analyzer --cmds "/root/owl/src/owl-wrapper ./bin/aes_test -m ENC -s 0 -f" --rand-cmd "/root/owl/src/owl-wrapper ./bin/aes_test -m ENC -s 0" -t 100

KEYS_DIR := bin/keys
KEY_SIZES := 512 1024 2048
PEM_FILES := $(addprefix $(KEYS_DIR)/private_key_, $(KEY_SIZES))


$(PEM_FILES): | $(KEYS_DIR)
	cd $(KEYS_DIR) && openssl genpkey -algorithm RSA -out private_key_$(subst private_key_,,$(notdir $@)).pem -pkeyopt rsa_keygen_bits:$(subst private_key_,,$(notdir $@))

$(KEYS_DIR):
	mkdir -p $(KEYS_DIR)

define generate_test_rsa_target
.PHONY: test_rsa_mp_$(1)

test_rsa_mp_$(1): $(PEM_FILES)
	/root/owl/src/owl_analyzer/target/release/owl_analyzer --cmds "/root/owl/src/owl-wrapper ./bin/rsa_test -f -m MP -b $(1)" --rand-cmd "/root/owl/src/owl-wrapper ./bin/rsa_test -m MP -b $(1)" -t 1

test_rsa_rns_$(1): $(PEM_FILES)
	/root/owl/src/owl_analyzer/target/release/owl_analyzer --cmds "/root/owl/src/owl-wrapper ./bin/rsa_test -f -m RNS -b $(1)" --rand-cmd "/root/owl/src/owl-wrapper ./bin/rsa_test -m RNS -b $(1)" -t 100
	
endef

$(foreach size,$(KEY_SIZES),$(eval $(call generate_test_rsa_target,$(size))))


clean:
	rm -f $(TARGET) $(OBJS) $(DEPS)
	make clean -C ./test

doc: $(SRC_FILES) doxygen.config
	doxygen doxygen.config

ifneq ($(MAKECMDGOALS), clean)
-include $(DEPS)
endif
