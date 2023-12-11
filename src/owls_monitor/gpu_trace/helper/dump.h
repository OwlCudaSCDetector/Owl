#ifndef OWLS_TRACE_DUMP_H
#define OWLS_TRACE_DUMP_H
#pragma once

#include <iostream>
#include <fstream>
#include <sstream>
#include <stdint.h>
#include <stdio.h>
#include <string>
#include <unistd.h>


#include "common.h"


const char *getTraceTypeStr(uint32_t type) {
  switch (type) {
  case TRACE_MEM_LOAD:
    return "TRACE_MEM_LOAD";
  case TRACE_MEM_STORE:
    return "TRACE_MEM_STORE";
  case TRACE_MEM_ATOMIC:
    return "TRACE_MEM_ATOMIC";

  case TRACE_FUNC_CALL:
    return "TRACE_FUNC_CALL";
  case TRACE_FUNC_RET:
    return "TRACE_FUNC_RET";
  case TRACE_BB_ACCESS:
    return "TRACE_BB_ACCESS";
  case TRACE_KERNEL_START:
    return "TRACE_KERNEL_START";
  }
  return "";
}

void dump_memory(const uint8_t* mem, uint32_t length) {
  printf("[host] dumping memory - start address : %p \n", mem);
  uint8_t new_line = 0;
  for(uint32_t i = 0; i< length; i++) {
    printf("0x%02x ", mem[i]);
    new_line += 1;
    if (new_line % 16 == 0) printf("\n");
  }
  printf("\n");
}


uint32_t trace_unpack(const uint8_t* buf) {
  common_info_t *common_info_ptr = (common_info_t *)buf;

  auto type = common_info_ptr->type & 0b01111111;
  
  switch (type) {
  case TRACE_MEM_LOAD:
  case TRACE_MEM_STORE:
  case TRACE_MEM_ATOMIC: {
    
    
    
    
    
    
    
    
    

    return sizeof(mem_record_t);
  }
  case TRACE_FUNC_CALL:
  case TRACE_FUNC_RET:
  case TRACE_BB_ACCESS:{
    

    
    
    
    

    return sizeof(bb_record_t);
  }

  case 6: {
        return 0x11;
    }
  case TRACE_KERNEL_START:{
      return sizeof(kernel_start_t);
      
  }

  case TRACE_CONTEXT_START:
  case TRACE_CONTEXT_STOP: {
    return 1;
  }


  default:
    fprintf(stderr, "deserialize error type : %d\n", type);
    dump_memory(buf, (uint32_t)sizeof(mem_record_t));
    exit(-1);
  };

  return 0;

}

#endif