#include "pipe.h"
#include <cstdio>
#include <cstdlib>
#include <fcntl.h>
#include <unistd.h>
#include "../debug.h"

int PIPE = -1;
uint64_t PIPE_SIZE = 1024 * 1024;

int set_pipe_size(uint64_t size) {
  if (PIPE == -1)
    return -1;
  return fcntl(PIPE, F_SETPIPE_SZ, size);
}

int open_pipe(char *pathname) {
  PIPE = open(pathname, O_WRONLY);
  if (PIPE == -1)
    return -1;
  set_pipe_size(PIPE_SIZE);
  return 0;
}

int close_pipe() {
  if (PIPE == -1)
    return -1;
  close(PIPE);
  return 0;
}

pthread_mutex_t send_data_mutex;
int send_data(uint8_t *data, size_t len) {
  if (PIPE == -1)
    return -1;
  pthread_mutex_lock(&send_data_mutex);
  bool debug = getenv("DEBUG");
  uint32_t offset = 0;
  do {
    
    
    

    size_t size = write(PIPE, data+offset, len-offset);
    if (size == 0xffffffffffffffff) {
      if (debug) BADF("send_data failed, get -1, try again");
    } else if ( size != len) {
      if (debug) BADF("send_data failed, hope: %lu, real: %lu", len, size);
      offset += size;
    } else {
      
      break;
    }
  }while(true);
  pthread_mutex_unlock(&send_data_mutex);
  return 0;
}

