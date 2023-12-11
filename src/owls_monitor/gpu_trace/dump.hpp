
#include "cfg.hpp"
#include "helper/json.hpp"
#include <vector>


enum JsonType { alloc, context, kernel };

inline const char* to_str(JsonType jt) {
    switch (jt) {
        case alloc:
            return "Alloc";
        case context:
            return "Context";
        case kernel:
            return "Kernel";
        default:
            return "Unknown";
        }
}

namespace owls {
    nlohmann::json to_json(vector<KernelTrace> kernel_traces);
    void json_to_file(nlohmann::json j, const char* filename);
    void json_to_pipe(nlohmann::json j, const char* pipename);
}