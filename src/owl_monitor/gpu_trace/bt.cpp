
#include "bt.hpp"


namespace owl {

FunctionInfo* extract_func_info(const char* str) {
    FunctionInfo *info = new FunctionInfo();
    
    const char* start = str;
    const char* end = strchr(start, '(');
    if (end != nullptr) {
        strncpy(info->filename, start, end - start);
        info->filename[end - start] = '\0';
    }

    
    start = strchr(str, '(');
    if (start != nullptr) {
        start++;
        end = strchr(start, '+');
        if (end != nullptr) {
            strncpy(info->functionname, start, end - start);
            info->functionname[end - start] = '\0';
        }
    }

    
    start = strchr(str, '+');
    if (start != nullptr) {
        start++;
        end = strchr(start, ')');
        if (end != nullptr) {
            char offsetStr[20];
            strncpy(offsetStr, start, end - start);
            offsetStr[end - start] = '\0';
            info->offset = std::stoull(offsetStr, nullptr, 16);
        }
    }

    
    start = strchr(str, '[');
    if (start != nullptr) {
        start++;
        end = strchr(start, ']');
        if (end != nullptr) {
            char addressStr[20];
            strncpy(addressStr, start, end - start);
            addressStr[end - start] = '\0';
            info->address = std::stoull(addressStr, nullptr, 16);
        }
    }

    return info;
}

}
