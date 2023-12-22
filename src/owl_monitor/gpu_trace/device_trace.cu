
#include <cstdint>
#include <cstdio>
#include <cstdlib>

#include <stdint.h>
#include <stdio.h>

#include "utils/utils.h"

#include "utils/channel.hpp"

#include "helper/common.h"

__device__ void dump_device_address(u8 *mem, u32 length) {
  printf("[device] dumping memory - start address : %p\n", mem);
  for (u32 i = 0; i < length; i++) {
    printf("0x%02x ", mem[i]);
  }
  printf("\n");
}

__device__ void dump_common_info(common_info_t &common_info) {
  printf(
      "[device] common_info : type:%d, smid:%d, warpid:%d, ctaid:{%d,%d,%d}\n",
      common_info.type, common_info.smid, common_info.warpid,
      common_info.ctaid.x, common_info.ctaid.y, common_info.ctaid.z);
}

extern "C" __device__ __noinline__ void __owl_device_trace(int pred, u32 type,
                                                            u64 bb_id, u64 desc,
                                                            u64 extra,
                                                            u64 pchannel_dev) {

  if (!pred) {
    return;
  }

  u32 lane_id = get_laneid();

  u32 active = __activemask();
  u32 active_mask = __ballot_sync(__activemask(), 1);
  u32 lowest = __ffs(active) - 1;

  u32 length = sizeof(common_info_t);
  u8 *cur_record = nullptr;

  common_info_t common_info;
  common_info.type = GPU_MODE((u8)type);
  common_info.smid = get_smid();
  common_info.warpid = get_warpid();

  common_info.ctaid.x = blockIdx.x;
  common_info.ctaid.y = blockIdx.y;
  common_info.ctaid.z = blockIdx.z;
  common_info.active = active;
  common_info.kernelid = u32(bb_id >> 32);
  common_info.bbid = bb_id & 0xffffffff;

  mem_record_t mem_record;
  bb_record_t bb_record;

  switch (type) {
  case TRACE_MEM_LOAD:
  case TRACE_MEM_STORE:
  case TRACE_MEM_ATOMIC: {
    mem_record.info = common_info;
    mem_record.type = (desc >> 48) & 0xffffffff;
    mem_record.size = (desc >> 32) & 0xffffffff;
    mem_record.offset = desc & 0xffffffff;

    for (u8 i = 0; i < warpSize; i++) {
      mem_record.addr[i] = __shfl_sync(active_mask, extra, i);
    }
    length = sizeof(mem_record_t);
    cur_record = (u8 *)&mem_record;
    break;
  }

  case TRACE_FUNC_CALL:
  case TRACE_FUNC_RET:
  case TRACE_BB_ACCESS: {
    bb_record.info = common_info;
    bb_record.instr_num = desc;
    length = sizeof(bb_record_t);
    cur_record = (u8 *)&bb_record;
    break;
  }
  }

  if (lane_id == lowest) {

    ChannelDev *channel_dev = (ChannelDev *)pchannel_dev;

    channel_dev->push(cur_record, length);
  }
}