#pragma once

#include <atomic>
#include <bits/types/FILE.h>
#include <cstdint>
#include <fcntl.h>
#include <libgen.h>
#include <linux/limits.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <thread>
#include <unistd.h>
#include <vector>


#define PIPE_MODE S_IWUSR | S_IRUSR | S_IRGRP | S_IROTH

extern "C" {
int send_data(uint8_t *data, size_t len);
int set_pipe_size(uint64_t size);
int open_pipe(char *pathname);
int close_pipe();

}