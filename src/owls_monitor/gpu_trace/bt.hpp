#ifndef BT_HPP
#define BT_HPP

#pragma once

#include <cstdint>
#include <cstring>
#include <iostream>
#include <vector>

using namespace std;

typedef struct {
  char filename[100];
  char functionname[1024];
  uint64_t offset;
  uint64_t address;
} FunctionInfo;

using BackTrace = vector<FunctionInfo *>;

namespace owls {
FunctionInfo *extract_func_info(const char *str);
}

#endif