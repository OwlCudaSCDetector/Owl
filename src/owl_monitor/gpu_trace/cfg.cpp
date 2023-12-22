#include "cfg.hpp"
#include <cassert>
#include <cstdlib>
#include <exception>
#include <iostream>
#include <memory>
#include <pthread.h>
#include <stdexcept>
#include <utility>

extern "C" bool debug;
extern "C" bool verbose;

bool operator<(const WarpKey &lhs, const WarpKey &rhs) {

  auto get_value = [](WarpKey wk) {
    return (u64)wk.first.x << 48 | (u64)wk.first.y << 32 |
           (u64)wk.first.z << 16 | (wk.second & 0xffff);
  };

  return get_value(lhs) < get_value(rhs);
}

bool operator<(const Direct &lhs, const Direct &rhs) {

  auto get_value = [](Direct wk) { return (u64)wk.begin << 32 | wk.end; };

  return get_value(lhs) < get_value(rhs);
}

bool operator<(const ControlFlow &lhs, const ControlFlow &rhs) {

  auto get_value = [](ControlFlow wk) { return (u64)wk.from << 32 | wk.to; };

  return get_value(lhs) < get_value(rhs);
}

void DCFGBuilder::record_bb_access(Dim cta_id, WarpID warp_id, NodeID node_id) {
    
    
  WarpKey warp_key = make_pair(cta_id, warp_id);

  
  if (warps.find(warp_key) == warps.end())
    warps[warp_key] = make_shared<WarpContext>(WarpContext{
        .id = warp_id,
        .pos = 0,
        .pre_node = make_shared<Node>(Node{.id = 0xffffffff}),
        .cur_node = make_shared<Node>(Node{.id = 0xffffffff}),
    });

  auto warp_context = warps[warp_key];
  auto pos = warp_context->pos;

  
  if (dcfg.nodes.find(node_id) == dcfg.nodes.end())
    dcfg.nodes[node_id] = make_shared<Node>(Node{
        .id = node_id,
    });

  auto node = dcfg.nodes[node_id];
  ControlFlow cf =
      ControlFlow{.from = warp_context->pre_node->id, .to = node_id};

  
  if (dcfg.nodes.find(warp_context->cur_node->id) != dcfg.nodes.end())
    ++dcfg.nodes[warp_context->cur_node->id]->ctrlFlowMap[cf];

  if (dcfg.nodes.find(warp_context->cur_node->id) != dcfg.nodes.end())
    ++dcfg.nodes[node_id]->fromMap[warp_context->cur_node->id];
  
  

  Direct direct = {
      .begin = warp_context->cur_node->id,
      .end = node_id,
  };

  
  if (dcfg.edges.find(direct) != dcfg.edges.end()) {
    auto edge = dcfg.edges[direct];
    auto position_map = edge->positions;
    position_map[pos] = (position_map.find(pos) != position_map.end())
                            ? position_map[pos] + 1
                            : 1;
  } else {
    dcfg.edges[direct] =
        make_shared<Edge>(Edge{.direct = direct, .positions = {{pos, 1}}});
  }

  warp_context->pos += 1;
  warp_context->pre_node = warp_context->cur_node;
  warp_context->cur_node = dcfg.nodes[node_id];
}

void DCFGBuilder::record_mem_access(Dim cta_id, WarpID warp_id, NodeID node_id,
                                    Addr offset, MemAccess access) {
  
  
  WarpKey warp_key = make_pair(cta_id, warp_id);

  
  if (warps.find(warp_key) == warps.end())
    BADF("warp %u not found.", warp_id);

  if (dcfg.nodes.find(node_id) == dcfg.nodes.end()) {
    
    
    record_bb_access(cta_id, warp_id, node_id);
  }

  if (node_id != warps[warp_key]->cur_node->id) {
    
    
    record_bb_access(cta_id, warp_id, node_id);
    
    
  }

  

  dcfg.nodes[node_id]->record_mem_access(warps[warp_key]->pos, offset, access);
}

void Node::record_mem_access(Pos pos, Addr offset, MemAccess ma) {
  
  auto itOffset = memAccessMap.find(offset);
  if (itOffset != memAccessMap.end()) {
    PosMemAccess &pmas = *(itOffset->second);
    
    auto itPos = pmas.find(pos);
    if (itPos != pmas.end()) {
      
      MemAccess &old_ma = *(itPos->second);

      
      for (auto m : ma) {
        auto type = m.first;
        auto &bmas = *(m.second);

        if (old_ma.find(type) == old_ma.end()) {
          old_ma[type] = m.second;
          continue;
        }
        auto &old_bmas = *(old_ma[type]);
        for (auto bma : bmas) {
          auto addr = bma.first;

          
          if (old_bmas.find(addr) != old_bmas.end()) {
            old_bmas[addr] += bma.second;
          } else {
            old_bmas[addr] = bma.second;
          }
        }
      }

    } else {
      pmas[pos] = make_shared<MemAccess>(ma);
    }
  } else {
    PosMemAccess pmas;
    pmas[pos] = make_shared<MemAccess>(ma);
    memAccessMap[offset] = make_shared<PosMemAccess>(pmas);
  }
}

KernelTrace KernelContext::collect() {
  return KernelTrace{.funcName = funcName,
                     .kernelID = kernelID,
                     .funcID = funcID,
                     .memPool = memPool,
                     .cfg = builder.dcfg,
                     .bt = bt};
}
