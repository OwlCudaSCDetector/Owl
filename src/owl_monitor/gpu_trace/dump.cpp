#include "dump.hpp"
#include "helper/json.hpp"
#include <iomanip>
#include <fstream>
#include <string>

using namespace nlohmann;

namespace owl {
json to_json(vector<KernelTrace> kernel_traces){
    json result = json::object();
    result["type"] = to_str(JsonType::kernel);

    json j = json::array();

    for (auto trace: kernel_traces) {
        string func_name = trace.funcName;
        u64 kernel_id = trace.kernelID;
        u64 func_id = trace.funcID;
        auto mem_pool = trace.memPool;
        auto dcfg = trace.cfg;
        auto bt = trace.bt;
        
        json j_mp = json::array();
        for (auto mem: mem_pool) {
            json j_meminfo = json::object();
            j_meminfo["addr"] = mem.addr;
            j_meminfo["size"] = mem.size;
            j_mp.push_back(j_meminfo);
        }

        json j_bt = json::array();
        for (auto fi: bt) {
            json j_funcinfo = json::object();
            j_funcinfo["file"] = fi->filename;
            j_funcinfo["func"] = fi->functionname;
            j_funcinfo["offset"] = fi->offset;
            j_funcinfo["addr"] = fi->address;
            j_bt.push_back(j_funcinfo);
        }

        json j_dcfg;
        j_dcfg["nodes"] = json::array();
        for (auto node: dcfg.nodes) {

            u64 node_id = node.first;
            auto accessMap = node.second->memAccessMap;
            auto fromMap = node.second->fromMap;
            auto controlflowMap = node.second->ctrlFlowMap;
            
            
            json j_imamap = json::array();
            for (auto ima: accessMap) {
                auto offset = ima.first;
                auto pma = *ima.second;
                json j_mamap = json::array();
                for (auto mam: pma) {
                    auto pos = mam.first;
                    auto mas = *mam.second; 

                    json j_access = json::array();
                    for (auto ma: mas) {

                        json j_ma = json::array();
                        for (auto m: *ma.second) {
                            auto addr = m.first;
                            auto count = m.second;

                            j_ma.push_back({
                                {"addr", addr},
                                {"count", count}
                            });
                        }

                        j_access.push_back({
                            {"type", ma.first},
                            {"memory", j_ma},
                        });
                    }
                    j_mamap.push_back({
                        {"pos", pos},
                        {"access", j_access},
                    }); 
                }
                j_imamap.push_back({
                    {"addr", offset},
                    {"data", j_mamap},
                });
            }

            json j_ctrls = json::array();
            
            if (controlflowMap.empty()) {
                for (auto fm: fromMap) {
                    json j_flow = {
                        {"from", (int32_t)fm.first},
                        {"to", -2},
                        {"num", fm.second}
                    };
                    j_ctrls.push_back(j_flow);
                }
            } else {
                for (auto cfm: controlflowMap) {
                    

                    auto flow = cfm.first;
                    json j_flow = {
                        {"from", (int32_t)flow.from},
                        {"to", (int32_t)flow.to},
                        {"num", cfm.second}
                    };

                    
                    j_ctrls.push_back(j_flow);
                }
            }

            j_dcfg["nodes"].push_back({
                {"id", node_id},
                {"mem_access", j_imamap},
                {"control_flow", j_ctrls}
            });
        }

        /*
        j_dcfg["edges"] = json::array();
        for (auto edge: dcfg.edges) {
            json j_edge = json::object();

            auto direct = edge.first;
            json j_direct = {
                {"start", direct.begin},
                {"end", direct.end},
            };

            j_edge["direct"] = j_direct;
        
            auto positions = edge.second->positions;

            j_edge["positions"]  = json::array();
            for (auto pos: positions) {
                j_edge["positions"].push_back({
                    {"pos", pos.first},
                    {"count", pos.second},
                });
            }

            j_dcfg["edges"].push_back(j_edge);
        }
        */
        
        j.push_back({
            {"name", func_name},
            {"id", kernel_id},
            {"ty", func_id},
            {"g", j_dcfg},
            {"bt", j_bt},
            {"mp", j_mp},
            });
        
    }

    result["data"] = j;
    return result;
}

void json_to_file(json j, const char* filename) {
    std::ofstream o(filename);
    o << std::setw(4) << j << std::endl;
    o.close();
}

void json_to_pipe(json j, const char* pipename) {
  std::ofstream outfifo(pipename, std::ofstream::binary);
  outfifo << j;
  outfifo.close();
}
}
