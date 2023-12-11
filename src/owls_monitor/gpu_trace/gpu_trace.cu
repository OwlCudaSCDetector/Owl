#include <algorithm>
#include <assert.h>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <exception>
#include <execinfo.h>
#include <fstream>
#include <functional>
#include <iostream>
#include <iterator>
#include <map>
#include <memory>
#include <ostream>
#include <sstream>
#include <stdint.h>
#include <stdio.h>
#include <string>
#include <sys/stat.h>
#include <unistd.h>
#include <unordered_map>
#include <unordered_set>
#include <utility>
#include <vector>

#include "helper/json.hpp"
#include "instr_types.h"
#include "nvbit.h"
#include "nvbit_tool.h"

#include "utils/channel.hpp"

#include "bt.hpp"
#include "cfg.hpp"
#include "dump.hpp"
#include "helper/common.h"
#include "helper/debug.h"
#include "helper/dump.h"
#include "helper/pipe/pipe.h"
#include "helper/shm.hpp"

using namespace std;

#define HEX(x) "0x" << setfill('0') << setw(16) << hex << (u64)x << dec

#define CHANNEL_SIZE (1l << 20)

#define KERNEL_INFO_FILE "kernel.info"

struct CTXstate {

  int id;

  ChannelDev *channel_dev;
  ChannelHost channel_host;
};

pthread_mutex_t mutex;

unordered_map<CUcontext, CTXstate *> ctx_state_map;

SharedMemory<AllocBasicInfo> sharedMemory(1024);

CTXstate *get_ctx_state(CUcontext ctx) {
  assert(ctx_state_map.find(ctx) != ctx_state_map.end());
  return ctx_state_map[ctx];
}

void set_ctx_state(CUcontext ctx, CTXstate *ctx_state) {
  assert(ctx_state_map.find(ctx) == ctx_state_map.end());
  ctx_state_map[ctx] = ctx_state;
}

bool skip_callback_flag = false;

int verbose = 0;
int debug = 0;
#define DEBUG(x) if (debug >= x)

u32 kernel_launch_id = 1;

bool doesRun = true;

map<string, vector<u64>> f_bb;
map<string, u64> f_id;

unordered_set<CUfunction> already_instrumented;
unordered_set<CUcontext> already_send_kernel_start;

map<KernelID, shared_ptr<KernelContext>> kernel_info;

void dump_address(u8 *mem, u32 length) {
  DEBUG(3) {
    WARNF("dumping memory - start address : %p", mem);
    for (u32 i = 0; i < length; i++) {
      printf("0x%02x ", mem[i]);
    }
    printf("\n");
  }
}

bool create_dir(const std::string &path) {
  struct stat info;
  if (stat(path.c_str(), &info) != 0) {
    if (mkdir(path.c_str(), S_IRWXU | S_IRWXG | S_IROTH | S_IXOTH) != 0) {

      return false;
    }
  } else if (!(info.st_mode & S_IFDIR)) {

    return false;
  }
  return true;
}


void send_context_start() {
  if (verbose)
    ACTF("Context Start");
}

void send_context_stop() {
  if (verbose)
    ACTF("Context Stop");
}

void send_kernel_start_data(u32 kernel_id, u64 func_id, Dim block, Dim thread) {

  if (verbose)
    ACTF("Kernel start : 0x%lx - %d", func_id, kernel_id);
}

void call_owls_device_trace(Instr *instr, trace_args_t ta) {
  nvbit_insert_call(instr, "__owls_device_trace", IPOINT_BEFORE);
  nvbit_add_call_arg_guard_pred_val(instr);
  nvbit_add_call_arg_const_val32(instr, ta.type);
  nvbit_add_call_arg_const_val64(instr, ta.bb_id);
  nvbit_add_call_arg_const_val64(instr, ta.desc);

  switch (ta.type) {
  case TRACE_MEM_LOAD:
  case TRACE_MEM_STORE: {
    nvbit_add_call_arg_mref_addr64(instr, ta.extra);
    break;
  }
  case TRACE_FUNC_CALL:
  case TRACE_BB_ACCESS:
  case TRACE_FUNC_RET: {
    nvbit_add_call_arg_const_val64(instr, ta.extra);
    break;
  }
  }
  nvbit_add_call_arg_const_val64(instr, ta.pchannel_dev);
}

void instrument_mem_access(CUcontext ctx, basic_block_t *bb, u64 bb_id) {
  auto get_instr_type = [](Instr *instr) {
    u8 type = TRACE_UNKNOWN;
    if (instr->isLoad())
      type = TRACE_MEM_LOAD;
    else if (instr->isStore())
      type = TRACE_MEM_STORE;
    return type;
  };

  for (auto instr : bb->instrs) {
    if (instr->getMemorySpace() == InstrType::MemorySpace::NONE ||
        instr->getMemorySpace() == InstrType::MemorySpace::CONSTANT) {
      continue;
    }

    unsigned long mref_idx = 0;
    for (int i = 0; i < instr->getNumOperands(); i++) {
      const InstrType::operand_t *op = instr->getOperand(i);

      if (op->type == InstrType::OperandType::MREF) {
        u8 type = get_instr_type(instr);
        if (type == TRACE_UNKNOWN)
          continue;

        trace_args_t ta = {
            .type = type,
            .bb_id = bb_id,
            .desc = (u64)instr->getMemorySpace() << 48 |
                    (u64)instr->getSize() << 32 | (u32)instr->getOffset(),
            .extra = mref_idx,
            .pchannel_dev = (u64)get_ctx_state(ctx)->channel_dev,
        };
        call_owls_device_trace(instr, ta);
        DEBUG(3)
        WARNF("Memory Access in Instr %d", instr->getIdx());
        mref_idx++;
      }
    }
  }
}

unordered_set<u32> func_entries;
void collect_func_entry(CUcontext ctx, CUfunction f) {
  func_entries.clear();
  func_entries.emplace(0);
  for (auto instr : nvbit_get_instrs(ctx, f)) {
    if (strcmp(instr->getOpcodeShort(), "CALL") == 0) {
      for (int i = 0; i < instr->getNumOperands(); i++) {
        const InstrType::operand_t *op = instr->getOperand(i);
        auto func_entry = op->u.imm_uint64.value;
        func_entries.emplace(func_entry);
      }
    }
  }
}

void instrument_bb_access(CUcontext ctx, basic_block_t *bb, u64 bb_id) {

  auto first_instr = bb->instrs[0];
  auto offset = first_instr->getOffset();

  u8 type = TRACE_BB_ACCESS;

  trace_args_t ta = {
      .type = type,
      .bb_id = bb_id,
      .desc = bb->instrs.size(),
      .pchannel_dev = (u64)get_ctx_state(ctx)->channel_dev,
  };
  call_owls_device_trace(first_instr, ta);

}

void instrument_basic_block(CUcontext ctx, basic_block_t *bb, u64 bb_id) {
  DEBUG(3) {
    WARNF("instrument_basic_block in %p , id : 0x%lx", bb, bb_id);
    for (auto i : bb->instrs) {
      i->print();
    }
  }
  instrument_bb_access(ctx, bb, bb_id);
  instrument_mem_access(ctx, bb, bb_id);
}

void instrument_function_if_needed(CUcontext ctx, CUfunction func) {
  auto related_functions = nvbit_get_related_functions(ctx, func);
  related_functions.push_back(func);

  for (auto f : related_functions) {
    auto func_name = nvbit_get_func_name(ctx, f);

    if (verbose)
      ACTF("instrument_function in function : %s", func_name);

    auto cfg = nvbit_get_CFG(ctx, f);

    string src;
    hash<string> h;

    u64 func_id =
        (f_id.find(func_name) != f_id.end()) ? f_id[func_name] : h(func_name);
    f_id[func_name] = func_id;

    vector<u64> bb_ids;
    for (auto bb : cfg.bbs) {

      u64 bb_id =
          ((u64)kernel_launch_id << 32) | (u32)(bb->instrs[0]->getOffset());

      bb_ids.push_back(bb_id);
      instrument_basic_block(ctx, bb, bb_id);

      /*


      char *fname, *dname; u32 line, line_stop;
      nvbit_get_line_info(ctx, f, bb->instrs[0]->getOffset(), &fname, &dname,
      &line); nvbit_get_line_info(ctx, f,
      bb->instrs[bb->instrs.size()-1]->getOffset(), &fname, &dname, &line_stop);
      bool is_valid_str = true;
      for (auto dname_c: string(dname)) {
        is_valid_str &= isprint(dname_c);
      }
      for (auto fname_c: string(fname)) {
        is_valid_str &= isprint(fname_c);
      }
      if (is_valid_str && verbose)
        ACTF("0x%lx: %s/%s:%u-%u",bb_id, dname, fname, line, line_stop);
      */
    }
    f_bb[func_name] = bb_ids;
  }
}

__global__ void flush_channel(ChannelDev *ch_dev) { ch_dev->flush(); }

BackTrace get_backtrace(size_t max_size = 100) {
  BackTrace bt;
  void *array[max_size];
  char **strings;
  int size;

  size = backtrace(array, max_size);
  if (size <= 0)
    return bt;

  strings = backtrace_symbols(array, size);
  if (strings != NULL) {

    for (int i = 6; i < size; i++) {

      bt.push_back(owls::extract_func_info(strings[i]));
    }
  }

  free(strings);

  return bt;
}

void nvbit_at_cuda_event(CUcontext ctx, int is_exit, nvbit_api_cuda_t cbid,
                         const char *name, void *params, CUresult *pStatus) {
  pthread_mutex_lock(&mutex);

  /* we prevent re-entry on this callback when issuing CUDA functions inside
   * this function */
  if (skip_callback_flag) {
    pthread_mutex_unlock(&mutex);
    return;
  }
  skip_callback_flag = true;

  if (cbid == API_CUDA_cuLaunchKernel_ptsz || cbid == API_CUDA_cuLaunchKernel) {
    cuLaunchKernel_params *p = (cuLaunchKernel_params *)params;

    cudaDeviceSynchronize();
    assert(cudaGetLastError() == cudaSuccess);

    if (!is_exit) {

      instrument_function_if_needed(ctx, p->f);

      int nregs = 0;
      CUDA_SAFECALL(
          cuFuncGetAttribute(&nregs, CU_FUNC_ATTRIBUTE_NUM_REGS, p->f));

      int shmem_static_nbytes = 0;
      CUDA_SAFECALL(cuFuncGetAttribute(
          &shmem_static_nbytes, CU_FUNC_ATTRIBUTE_SHARED_SIZE_BYTES, p->f));

      const char *func_name = nvbit_get_func_name(ctx, p->f);
      u64 pc = nvbit_get_func_addr(p->f);

      nvbit_set_at_launch(ctx, p->f, &kernel_launch_id, sizeof(u32));

      doesRun = true;

      auto block = Dim{p->gridDimX, p->gridDimY, p->gridDimZ};
      auto thread = Dim{p->blockDimX, p->blockDimY, p->blockDimZ};

      MemPool abi = sharedMemory.parse();

      kernel_info[kernel_launch_id] =
          make_shared<KernelContext>(KernelContext{.funcName = func_name,
                                                   .kernelID = kernel_launch_id,
                                                   .funcID = f_id[func_name],
                                                   .memPool = abi,
                                                   .builder = DCFGBuilder(),
                                                   .bt = get_backtrace()});
      send_kernel_start_data(kernel_launch_id++, f_id[func_name], block,
                             thread);

      bool flag = !getenv("NOINSTR");
      nvbit_enable_instrumented(ctx, p->f, flag);

      OKF("CTX 0x%016lx - LAUNCH - Kernel pc 0x%016lx - Kernel "
          "name %s - grid launch id %ld - grid size %d,%d,%d - block "
          "size %d,%d,%d - nregs %d - shmem %d - cuda stream id %ld",
          (u64)ctx, pc, func_name, kernel_launch_id, p->gridDimX, p->gridDimY,
          p->gridDimZ, p->blockDimX, p->blockDimY, p->blockDimZ, nregs,
          shmem_static_nbytes + p->sharedMemBytes, (u64)p->hStream);
    } else {
      nvbit_enable_instrumented(ctx, p->f, false);
    }
  }
  skip_callback_flag = false;
  pthread_mutex_unlock(&mutex);
}

void deserialize_trace(u8 *buffer, u32 len) {
  u32 offset = 0;
  u8 *cur_buffer;
  u8 type;
  while (offset < len) {
    cur_buffer = buffer + offset;

    if (!(*cur_buffer & TYPE_GPU))
      type = 0xff;
    else
      type = *cur_buffer & ~TYPE_GPU;

    switch (type) {

    case TRACE_BB_ACCESS: {
      bb_record_t *r = (bb_record_t *)cur_buffer;

      kernel_info[r->info.kernelid]->builder.record_bb_access(
          r->info.ctaid, r->info.warpid, r->info.bbid);

      offset += sizeof(bb_record_t);
      break;
    }

    case TRACE_MEM_LOAD:
    case TRACE_MEM_STORE:
    case TRACE_MEM_ATOMIC: {
      mem_record_t *r = (mem_record_t *)cur_buffer;

      auto bma = make_shared<BasicMemAccess>();

      for (int i = 0; i < 32; i++) {
        if (!((r->info.active >> i) % 2))
          continue;
        auto addr = r->addr[i];
        (*bma)[addr] =
            ((*bma).find(addr) != (*bma).end()) ? (*bma)[addr] + 1 : 1;
      }

      MemAccess ma;
      ma[InstrType::MemorySpaceStr[r->type]] = bma;

      try {

        kernel_info[r->info.kernelid]->builder.record_mem_access(
            r->info.ctaid, r->info.warpid, r->info.bbid, r->offset, ma);
      } catch (const char *&e) {
        auto funcID = kernel_info[r->info.kernelid]->funcID;
        for (auto f : f_id) {
          if (f.second == funcID) {
            auto funcName = f.first;
            BADF("record mem access failed. %s", e);
            cerr << "func name : " << funcName << endl;
            break;
          }
        }

        exit(EXIT_FAILURE);
      }

      offset += sizeof(mem_record_t);
      break;
    }
    default:
      BADF("deserialize error! unsupported type : %s", getTraceTypeStr(type));
    }
  }
}

void *recv_thread_fun(void *args) {
  CUcontext ctx = (CUcontext)args;

  pthread_mutex_lock(&mutex);

  CTXstate *ctx_state = get_ctx_state(ctx);

  ChannelHost *ch_host = &ctx_state->channel_host;
  pthread_mutex_unlock(&mutex);
  char *recv_buffer = (char *)malloc(CHANNEL_SIZE);

  while (true) {
    u32 num_recv_bytes = ch_host->recv(recv_buffer, CHANNEL_SIZE);
    if (num_recv_bytes > 0) {

      deserialize_trace((u8 *)recv_buffer, num_recv_bytes);
    } else if (!doesRun) {
      break;
    }
  }
  free(recv_buffer);
  return NULL;
}

void DumpJson(nlohmann::json j) {
  if (char *dirname = getenv("OWL_TRACE")) {
    create_dir(dirname);

    char *kernel_filename =
        new char[strlen(dirname) + strlen("/kernel.json") + 1];
    strcpy(kernel_filename, dirname);
    strcat(kernel_filename, "/kernel.json");
    owls::json_to_file(j, kernel_filename);
  }
  if (char *pipename = getenv("OWLS_PIPE2"))
    owls::json_to_pipe(j, pipename);
  if (char *filename = getenv("OWLS_FILE"))
    owls::json_to_file(j, filename);
}

void collect_kernel_trace() {
  DEBUG(1) ACTF("staring collect kernel trace");
  cout << flush;
  vector<KernelTrace> kernel_traces;
  int idx = 0;
  for (auto iter = kernel_info.begin(); iter != kernel_info.end();) {
    auto trace = iter->second->collect();
    kernel_traces.push_back(trace);

    if (verbose)
      ACTF("collect kernel 0x%lx-%lu", trace.funcID, trace.kernelID);

    iter->second.reset();
    kernel_info.erase(iter++);
  }

  DEBUG(1) ACTF("staring dump kernel trace");
  nlohmann::json j = owls::to_json(kernel_traces);

  DumpJson(j);
}

void nvbit_at_init() {
  setenv("CUDA_MANAGED_FORCE_DEVICE_ALLOC", "1", 1);
  GET_VAR_INT(verbose, "VERBOSE", 0, "Enable verbosity inside the tool");
  GET_VAR_INT(debug, "DEBUG", 0, "Enable debug info inside the tool");
  if (!getenv("NOBANNER")) {
    string pad(100, '-');
    printf("%s\n", pad.c_str());
  }

  pthread_mutexattr_t attr;
  pthread_mutexattr_init(&attr);
  pthread_mutexattr_settype(&attr, PTHREAD_MUTEX_RECURSIVE);
  pthread_mutex_init(&mutex, &attr);
}

void nvbit_at_ctx_init(CUcontext ctx) {
  pthread_mutex_lock(&mutex);
  OKF("STARTING CONTEXT %p", ctx);

  if (!sharedMemory.attach()) {
    std::cerr << "Failed to attach shared memory!" << std::endl;
    return;
  }

  send_context_start();
  CTXstate *ctx_state = new CTXstate;

  set_ctx_state(ctx, ctx_state);

  cudaMallocManaged(&ctx_state->channel_dev, sizeof(ChannelDev));
  ctx_state->channel_host.init((int)ctx_state_map.size() - 1, CHANNEL_SIZE,
                               ctx_state->channel_dev, recv_thread_fun, ctx);
  nvbit_set_tool_pthread(ctx_state->channel_host.get_thread());
  pthread_mutex_unlock(&mutex);
}

void nvbit_at_ctx_term(CUcontext ctx) {
  pthread_mutex_lock(&mutex);
  skip_callback_flag = true;

  CTXstate *ctx_state = get_ctx_state(ctx);

  flush_channel<<<1, 1>>>(ctx_state->channel_dev);

  cudaDeviceSynchronize();
  doesRun = false;
  assert(cudaGetLastError() == cudaSuccess);

  ofstream f_info;

  ctx_state->channel_host.destroy(false);
  cudaFree(ctx_state->channel_dev);
  skip_callback_flag = false;
  delete ctx_state;

  sharedMemory.detach();

  send_context_stop();
  collect_kernel_trace();
  OKF("TERMINATING CONTEXT %p", ctx);

  pthread_mutex_unlock(&mutex);
}
