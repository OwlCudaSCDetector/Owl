#pragma once
#ifndef COMMON_H
#define COMMON_H

#include <cstdint>
#include <stdint.h>

typedef uint8_t u8;
typedef uint16_t u16;
typedef uint32_t u32;
typedef uint64_t u64;


typedef struct {
  u8 *allocs;
  u8 *commits;
  u8 *records;
  u32 slot_size;
} traceinfo_t;

#pragma pack(1)

typedef struct {
  u32 x, y, z;
} Dim;

typedef struct common_info_t {
  u8 type;
  u32 smid;
  u16 warpid;
  Dim ctaid;
  u32 kernelid;
  u32 bbid;
  u32 active;
} common_info_t;

typedef struct {
  common_info_t info;
  u32 type;
  u32 size;
  u32 offset;
  u64 addr[32];
} mem_record_t;

typedef struct {
  common_info_t info;
  u32 instr_num;
} bb_record_t;

typedef struct {
  common_info_t info;
  u32 loopid;
  u32 iterid;
} loop_record_t;

typedef struct {
  u8 type;
  u64 kernelid;
  u64 funcid;
  Dim Block;
  Dim Thread;
} kernel_start_t;






#pragma pack()

enum TRACE_TYPE {
  TRACE_KERNEL_START = 0,

  TRACE_MEM_LOAD = 1,
  TRACE_MEM_STORE = 2,
  TRACE_MEM_ATOMIC = 3,

  TRACE_FUNC_CALL = 4,
  TRACE_BB_ACCESS = 5,

  TRACE_FUNC_RET = 8,

  
  

  TRACE_CONTEXT_START = 0x7E,
  TRACE_CONTEXT_STOP = 0x7F,
  TRACE_UNKNOWN =  0x70,
};


#define TYPE_GPU 0b10000000
#define GPU_MODE(x) (x | TYPE_GPU) 

typedef struct {
  u8 type;
  u64 bb_id;
  u64 desc;
  u64 extra;
  u64 pchannel_dev;
} trace_args_t;


#define PREFIX "GPUTrace"


#endif
