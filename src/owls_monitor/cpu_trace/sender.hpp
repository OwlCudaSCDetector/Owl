#include "json.hpp"
#include <fstream>
#include <iomanip>

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

#define OWLS_PIPE "OWLS_PIPE2"
#define OWLS_FILE "OWLS_FILE"
#define OWL_TRACE "OWL_TRACE"

namespace owls {
void json_to_file(nlohmann::json j, const char *filename) {
  std::ofstream o(filename);
  o << std::setw(4) << j << std::endl;
  o.close();
}
void json_to_pipe(nlohmann::json j, const char *pipename) {
  std::ofstream outfifo(pipename, std::ofstream::binary);
  outfifo << j;
  outfifo.close();
}
} // namespace owls