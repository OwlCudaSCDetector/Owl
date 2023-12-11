#include <cassert>
#include <charconv>
#include <cstddef>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <iostream>
#include <memory>
#include <stack>
#include <string>
#include <sys/ipc.h>
#include <sys/shm.h>
#include <sys/stat.h>
#include <types.h>
#include <unistd.h>
#include <vector>

#define PREFIX "CPUTrace"

#include "control_manager.H"
#include "debug.h"
#include "instlib.H"
#include "json.hpp"
#include "pin.H"
#include "regvalue_utils.h"
#include "sender.hpp"
#include "shm.hpp"
#include "types_base.PH"
#include "types_vmapi.PH"

using namespace CONTROLLER;
using namespace INSTLIB;
using namespace std;
using namespace nlohmann;

KNOB<string> KnobOutputFile(KNOB_MODE_WRITEONCE, "pintool", "o",
                            "alloctrace.out", "trace file");

KNOB<string> KnobExecutableName(KNOB_MODE_WRITEONCE, "pintool", "exe",
                                "program.exe", "Specify the executable name");

KNOB<int> KnobDebug(KNOB_MODE_WRITEONCE, "pintool", "debug", "0",
                    "Enable debugging output.");

#define DEBUG(x) if (getenv("DEBUG") && atoi(getenv("DEBUG")) >= x)

/*
**********************************utils-function************************************
*/

VOID PrintIns(INS ins) {
  ADDRINT address = INS_Address(ins);
  const char *disassembly = INS_Disassemble(ins).c_str();

  ACTF("Instruction at address 0x%lx: %s", address, disassembly);
}

VOID GetImgName(IMG img, VOID *v) {
  const char *imgName = IMG_Name(img).c_str();
  ACTF("Executable Image Name: %s", imgName);
}

VOID GetRoutineName(RTN rtn, VOID *v) {
  const char *routineName = RTN_Name(rtn).c_str();
  ACTF("Routine Name: %s", routineName);
}

template <typename T> T GetAddrValue(ADDRINT argPtr) {
  T value;
  if (PIN_SafeCopy(&value, reinterpret_cast<T *>(argPtr), sizeof(T)) !=
      sizeof(T)) {
    BADF("Failed to read value at address: %lx", argPtr);
    exit(-1);
  }
  return value;
}

bool create_dir(const char *path) {
  struct stat info;
  if (stat(path, &info) != 0) {
    if (mkdir(path, S_IRWXU | S_IRWXG | S_IROTH | S_IXOTH) != 0) {

      return false;
    }
  } else if (!(info.st_mode & S_IFDIR)) {

    return false;
  }
  return true;
}

void DumpJson(json j) {
  if (char *pipename = getenv(OWLS_PIPE))
    owls::json_to_pipe(j, pipename);
  if (char *filename = getenv(OWLS_FILE))
    owls::json_to_file(j, filename);
}

typedef struct {
  uint64_t addr;
  uint64_t size;
} AllocBasicInfo;

vector<AllocBasicInfo *> dev_mem;

SharedMemory<AllocBasicInfo> sharedMemory(1024);

struct AllocInfo {
  CHAR *name;
  ADDRINT host_ptr;
  ADDRINT cur_ptr;
  ADDRINT cur_size;

  json to_json_lite() const {
    json j = json::object();
    j["name"] = name;
    j["addr"] = cur_ptr;
    j["size"] = cur_size;
    return j;
  }

  json to_json() const {
    json result = json::object();
    result["type"] = to_str(JsonType::alloc);
    result["data"] = to_json_lite();

    return result;
  }

  void dump() const { DumpJson(this->to_json()); }
};

VOID DeviceAllocBefore(CHAR *name, ADDRINT ptr, ADDRINT size, AllocInfo *ai) {
  ai->name = name;
  ai->cur_ptr = ptr;
  ai->cur_size = size;
}

VOID DeviceAllocAfter(AllocInfo *ai) {

  ai->cur_ptr = GetAddrValue<UINT64>(ai->cur_ptr);

  DEBUG(2)
  WARNF("%s Allocation in address: 0x%lx, size: 0x%lx", ai->name, ai->cur_ptr,
        ai->cur_size);

  dev_mem.push_back(
      new AllocBasicInfo{.addr = ai->cur_ptr, .size = ai->cur_size});
  sharedMemory.write(dev_mem);
}

/**
  cudaHostAlloc(&RecordsHost, SLOTS_NUM * SLOTS_SIZE, cudaHostAllocMapped);
  cudaHostGetDevicePointer(&RecordsDevice, RecordsHost, 0);
*/
map<ADDRINT, ADDRINT> host_2_dev;

vector<AllocInfo *> host_alloc_mem;

VOID HostAllocBefore(CHAR *name, ADDRINT host_ptr, ADDRINT size,
                     AllocInfo *ai) {
  ai->name = name;
  ai->host_ptr = host_ptr;
  ai->cur_size = size;
}

VOID HostAllocAfter(AllocInfo *ai) {

  host_alloc_mem.push_back(ai);
  DEBUG(2)
  WARNF("%s Allocation in host address: 0x%lx, size: 0x%lx", ai->name,
        ai->cur_ptr, ai->cur_size);
  ai->host_ptr = GetAddrValue<UINT64>(ai->host_ptr);
}

/**
  cudaHostAlloc(&RecordsHost, SLOTS_NUM * SLOTS_SIZE, cudaHostAllocMapped);
  cudaHostGetDevicePointer(&RecordsDevice, RecordsHost, 0);
*/
AllocInfo *ai;
VOID HostGetDevicePointerBefore(ADDRINT dev_ptr, ADDRINT host_ptr) {
  if (host_2_dev.find(host_ptr) != host_2_dev.end())
    return;

  bool finded = false;

  for (auto a : host_alloc_mem) {
    if (a->host_ptr == host_ptr) {
      a->cur_ptr = dev_ptr;
      ai = a;
      finded = true;
    }
  }

  assert(finded == true);
}

VOID HostGetDevicePointerAfter() {

  ai->cur_ptr = GetAddrValue<UINT64>(ai->cur_ptr);

  DEBUG(2)
  WARNF("cudaHostGetDevicePointer in dev address: 0x%lx, size: 0x%lx",
        ai->cur_ptr, ai->cur_size);

  host_2_dev[ai->host_ptr] = ai->cur_ptr;

  dev_mem.push_back(
      new AllocBasicInfo{.addr = ai->cur_ptr, .size = ai->cur_size});

  sharedMemory.write(dev_mem);
  ai = nullptr;
}

VOID CudaFreeBefore(ADDRINT dev_ptr) {

  DEBUG(2)
  WARNF("cudaFree in dev address: 0x%lx", dev_ptr);

  for (auto it = dev_mem.begin(); it != dev_mem.end();) {
    if ((*it)->addr == dev_ptr) {
      delete *it;
      it = dev_mem.erase(it);
      break;
    } else {
      ++it;
    }
  }
  sharedMemory.write(dev_mem);
}

VOID TraceMemoryPool(IMG img, VOID *v) {
  vector<string> DeviceAllocFunction = {"cudaMalloc", "cudaMallocAsync",
                                        "cudaMallocFromPoolAsync",
                                        "cudaMallocManaged"};

  vector<string> HostAllocFunction = {"cudaHostAlloc", "cudaMallocHost"};

  vector<string> NoImplFunction = {"cudaMallocPitch", "cudaMallocArray",
                                   "cudaMalloc3D", "cudaMalloc3DArray",
                                   "cudaMallocMipmappedArray"};

  for (const auto &f : DeviceAllocFunction) {
    const char *cstr = f.c_str();
    CHAR *fname = strdup(cstr);

    RTN devMallocRtn = RTN_FindByName(img, fname);
    if (RTN_Valid(devMallocRtn)) {
      RTN_Open(devMallocRtn);

      AllocInfo *ai = new AllocInfo();
      RTN_InsertCall(devMallocRtn, LEVEL_VM::IPOINT_BEFORE,
                     (AFUNPTR)DeviceAllocBefore, IARG_ADDRINT, fname,
                     IARG_FUNCARG_ENTRYPOINT_VALUE, 0,
                     IARG_FUNCARG_ENTRYPOINT_VALUE, 1, IARG_PTR, ai, IARG_END);

      RTN_InsertCall(devMallocRtn, LEVEL_VM::IPOINT_AFTER,
                     (AFUNPTR)DeviceAllocAfter, IARG_PTR, ai, IARG_END);

      RTN_Close(devMallocRtn);
      free(ai);
    }
  }

  for (const auto &f : HostAllocFunction) {
    const char *cstr = f.c_str();
    CHAR *fname = strdup(cstr);

    RTN hostMallocRtn = RTN_FindByName(img, fname);
    if (RTN_Valid(hostMallocRtn)) {
      RTN_Open(hostMallocRtn);

      AllocInfo *ai = new AllocInfo();
      RTN_InsertCall(hostMallocRtn, LEVEL_VM::IPOINT_BEFORE,
                     (AFUNPTR)HostAllocBefore, IARG_ADDRINT, fname,
                     IARG_FUNCARG_ENTRYPOINT_VALUE, 0,
                     IARG_FUNCARG_ENTRYPOINT_VALUE, 1, IARG_PTR, ai, IARG_END);

      RTN_InsertCall(hostMallocRtn, LEVEL_VM::IPOINT_AFTER,
                     (AFUNPTR)HostAllocAfter, IARG_PTR, ai, IARG_END);

      RTN_Close(hostMallocRtn);
      free(ai);
    }
  }

  {
    char hostGetDevPtrName[] = "cudaHostGetDevicePointer";
    RTN hostGetDevPtrRtn = RTN_FindByName(img, hostGetDevPtrName);
    if (RTN_Valid(hostGetDevPtrRtn)) {
      RTN_Open(hostGetDevPtrRtn);

      RTN_InsertCall(hostGetDevPtrRtn, LEVEL_VM::IPOINT_BEFORE,
                     (AFUNPTR)HostGetDevicePointerBefore,
                     IARG_FUNCARG_ENTRYPOINT_VALUE, 0,
                     IARG_FUNCARG_ENTRYPOINT_VALUE, 1, IARG_END);

      RTN_InsertCall(hostGetDevPtrRtn, LEVEL_VM::IPOINT_AFTER,
                     (AFUNPTR)HostGetDevicePointerAfter, IARG_END);

      RTN_Close(hostGetDevPtrRtn);
      free(ai);
    }
  }

  {
    char cudaFreeName[] = "cudaFree";
    RTN cudaFreeRtn = RTN_FindByName(img, cudaFreeName);
    if (RTN_Valid(cudaFreeRtn)) {
      RTN_Open(cudaFreeRtn);

      RTN_InsertCall(cudaFreeRtn, LEVEL_VM::IPOINT_BEFORE,
                     (AFUNPTR)CudaFreeBefore, IARG_FUNCARG_ENTRYPOINT_VALUE, 0,
                     IARG_END);

      RTN_Close(cudaFreeRtn);
      free(ai);
    }
  }
}

INT32 Usage() {
  cerr << endl << KNOB_BASE::StringKnobSummary() << endl;
  return -1;
}

VOID Fini(INT32 code, VOID *v) {

  sharedMemory.detach();
  sharedMemory.remove();
}

int main(int argc, char *argv[]) {
  PIN_InitSymbols();
  if (PIN_Init(argc, argv))
    return Usage();

  if (!sharedMemory.create()) {
    std::cerr << "Failed to create shared memory!" << std::endl;
    return 1;
  }

  IMG_AddInstrumentFunction(TraceMemoryPool, 0);

  PIN_AddFiniFunction(Fini, nullptr);

  PIN_StartProgram();

  return 0;
}