#ifndef CFG_HPP
#define CFG_HPP

#include "helper/common.h"
#include "helper/debug.h"
#include "bt.hpp"

#include <cstdint>
#include <cstdio>
#include <map>
#include <memory>
#include <pthread.h>
#include <string>
#include <unordered_map>
#include <utility>
#include <vector>

using namespace std;

using WarpID = u16;
using BBID = u32;
using NodeID = BBID;
using Pos = u32;

typedef struct {
  NodeID begin;
  NodeID end;
} Direct;


using Addr = u64;
using Count = u64;
using BasicMemAccess =  map<Addr, Count>;

using Type = const char*;
using MemAccess =  map<Type, shared_ptr<BasicMemAccess>>;

using PosMemAccess = map<Pos, shared_ptr<MemAccess>>;

using InstrMemAccess =  map<Addr, shared_ptr<PosMemAccess>>;

typedef struct {
  NodeID from;
  NodeID to;
} ControlFlow;


typedef struct {
  NodeID id;
  map<NodeID, Count> fromMap;
  map<ControlFlow, Count> ctrlFlowMap;

  InstrMemAccess memAccessMap;

  void Node(int _id = 0xffffffff){id = _id;}
  void record_mem_access(Pos pos, Addr offset, MemAccess ma);
} Node;


typedef struct {
  Direct direct;
  map<Pos, Count> positions;
} Edge;

typedef struct {
  map<NodeID, shared_ptr<Node>> nodes;
  map<Direct, shared_ptr<Edge>> edges;
} DCFG;






typedef struct {
  uint64_t addr;
  uint64_t size;
} AllocBasicInfo;

using MemPool = vector<AllocBasicInfo>;


using KernelID = u64;
using FuncID = u64;

typedef struct {
  string funcName;
  KernelID kernelID;
  FuncID funcID;
  MemPool memPool;
  DCFG cfg;
  BackTrace bt;
} KernelTrace;





using WarpKey = pair<Dim, WarpID> ;

class DCFGBuilder {  

public:
  DCFGBuilder() {
    dcfg.nodes = map<NodeID, shared_ptr<Node>>(), 
    dcfg.edges = map<Direct, shared_ptr<Edge>>(),
    warps = map<WarpKey, shared_ptr<WarpContext>>();
  }

  void record_bb_access(Dim cta, WarpID warp, NodeID node);
  void record_mem_access(Dim cta, WarpID warp, NodeID node, Addr offset, MemAccess access);


  DCFG dcfg;

private:
  
  typedef struct {
    WarpID id;
    u64 pos;
    shared_ptr<Node> pre_node;
    shared_ptr<Node> cur_node;
    
    
  } WarpContext;


  map<WarpKey, shared_ptr<WarpContext>> warps;

};



typedef struct {
  string funcName;
  KernelID kernelID;
  FuncID funcID;
  MemPool memPool;
  DCFGBuilder builder;
  BackTrace bt;

  KernelTrace collect();
} KernelContext;


void collect_kernel_trace();

#endif