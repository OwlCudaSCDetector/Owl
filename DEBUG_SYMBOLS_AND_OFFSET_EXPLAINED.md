# Debug Symbols and Backtrace Offset Explained

## What Are Debug Symbols?

**Debug symbols** are extra metadata embedded in compiled binaries that map machine code back to human-readable source code.

### What Debug Symbols Contain:

1. **Function names** - The actual names from source code (e.g., `main`, `AES_encrypt`, `cudaLaunchKernel`)
2. **Variable names** - Local and global variable identifiers
3. **Source file names** - Which `.c` or `.cpp` file the code came from
4. **Line numbers** - Which line in source code corresponds to each instruction
5. **Type information** - Data structures, classes, etc.

### Binary With vs Without Debug Symbols:

#### WITH Debug Symbols (your aes_test):
```bash
$ file aes_test
ELF 64-bit LSB pie executable, x86-64, version 1 (SYSV), 
dynamically linked, interpreter /lib64/ld-linux-x86-64.so.2, 
BuildID[sha1]=48f77f016e78d4ed3b7d3aeac1ddbbd89369b56d, 
for GNU/Linux 3.2.0, with debug_info, not stripped
                      ^^^^^^^^^^^^^^^^^ ^^^^^^^^^^^^^
                      Has debug info!   Symbols intact!
```

#### WITHOUT Debug Symbols (stripped):
```bash
$ file aes_test_stripped
ELF 64-bit LSB pie executable, x86-64, version 1 (SYSV),
dynamically linked, interpreter /lib64/ld-linux-x86-64.so.2,
for GNU/Linux 3.2.0, stripped
                     ^^^^^^^^
                     No symbols!
```

---

## Real Examples from Your kernel.json

Let's look at your actual backtrace:

### Example 1: WITH Debug Symbols (libcuda.so)

```json
{
  "addr": 140659222238917,
  "file": "/usr/lib/x86_64-linux-gnu/libcuda.so.1",
  "func": "cuLaunchKernel",  ← Function name found!
  "offset": 53
}
```

**Why `func` has a value:**
- `libcuda.so.1` contains exported symbols (public API)
- Even without full debug info, shared libraries export function names
- The tracer can look up the symbol table and find `cuLaunchKernel`

### Example 2: WITHOUT Debug Symbols (aes_test)

```json
{
  "addr": 110918256501076,
  "file": "/workspace/example/crypt-examples/libgpucrypto/bin/aes_test",
  "func": "",  ← Empty! No function name found
  "offset": 102740
}
```

**Why `func` is empty:**
- Even though your binary says "not stripped", the tracer couldn't resolve this specific address
- This could be:
  - An inlined function (no separate symbol)
  - Compiler-generated code (thunk, trampoline)
  - Static/local function (not in dynamic symbol table)
  - Address resolution failed

### Example 3: WITH Debug Symbols (libcudart.so)

```json
{
  "addr": 140659321384118,
  "file": "/usr/local/cuda/targets/x86_64-linux/lib/libcudart.so.11.0",
  "func": "cudaLaunchKernel",  ← Function name found!
  "offset": 598
}
```

---

## Understanding the "offset" Field

The **offset** is the number of bytes from the **start of the binary file** to the instruction address.

### How It Works:

When a program is loaded into memory:

```
Disk (File):                    Memory (Runtime):
┌─────────────────┐            ┌─────────────────┐
│ ELF Header      │            │                 │
│ Program Headers │            │                 │
├─────────────────┤            │                 │
│ .text (code)    │  ─────>    │ Code at 0x7f... │
│   offset 4096   │  loaded    │                 │
│   size 102740   │            │                 │
├─────────────────┤            └─────────────────┘
│ .data           │
│ .rodata         │
└─────────────────┘
```

### Offset Calculation Example:

Let's use this backtrace entry:
```json
{
  "addr": 110918256501076,
  "file": "/workspace/.../bin/aes_test",
  "func": "",
  "offset": 102740
}
```

**Step-by-step:**

1. **Runtime address**: `110918256501076` (0x64F8E1F55DD4 in hex)
2. **Binary base address**: This is where the binary was loaded in memory
3. **File offset**: `102740` bytes (0x191D4 in hex)

**The calculation:**
```
offset = runtime_address - base_address
102740 = 110918256501076 - 110918256398336
```

**What this means:**
- The instruction that was executing is located at byte **102740** in the `aes_test` file
- If you open `aes_test` in a hex editor and go to offset 102740, you'll see the machine code
- If you disassemble the binary at offset 102740, you'll see the actual assembly instruction

---

## How the Tracer Gets This Information

### Step 1: Capture Stack Trace

When the GPU kernel is launched, the tracer walks the CPU call stack:

```c
// Pseudocode of what owl-wrapper does
void capture_backtrace() {
    void* frames[100];
    int depth = backtrace(frames, 100);  // Get return addresses
    
    for (int i = 0; i < depth; i++) {
        BacktraceEntry entry;
        entry.addr = (uint64_t)frames[i];  // Runtime address
        
        // Find which binary this address belongs to
        Dl_info info;
        if (dladdr(frames[i], &info)) {
            entry.file = info.dli_fname;      // Binary path
            entry.func = info.dli_sname;      // Function name (if available)
            entry.offset = (char*)frames[i] - (char*)info.dli_fbase;
        }
        
        backtrace_data.push_back(entry);
    }
}
```

### Step 2: Symbol Resolution

The `dladdr()` function (on Linux) does:

1. Checks `/proc/self/maps` to find which shared library contains the address
2. Reads the ELF symbol table from that library
3. Finds the nearest symbol before the address
4. Returns symbol name if found, or NULL if not

**Example with symbols:**
```
Symbol table:
0x1000: main
0x1200: AES_encrypt
0x1500: cudaLaunchKernel
0x1800: another_function

Address 0x1520 → Nearest symbol: cudaLaunchKernel at 0x1500
                 Offset within function: 0x20 (32 bytes)
```

**Example without symbols:**
```
Symbol table:
(empty or stripped)

Address 0x1520 → No symbol found
                 func = ""
                 offset = entire offset from binary base
```

---

## Practical Example: Using the Offset

### Finding the Code

You can use the offset to locate the exact code:

```bash
# Method 1: Disassemble at offset
objdump -d aes_test | grep "191d4:"

# Method 2: Use addr2line (if debug info available)
addr2line -e aes_test -f -a 102740

# Method 3: GDB
gdb aes_test
(gdb) disassemble 0x191d4
```

### Example Output:

```assembly
00191d0:  e8 1b fe ff ff    call   18ff0 <cudaLaunchKernel@plt>
00191d5:  48 83 c4 08       add    rsp,0x8
00191d9:  5b                pop    rbx
00191da:  5d                pop    rbp
```

**This tells you:**
- At offset 102740 (0x191d4), there's a call to `cudaLaunchKernel`
- This is in your AES encryption function
- This is the exact point where the GPU kernel was launched

---

## Why Different Files Have Different Information

### System Libraries (libcuda.so, libcudart.so):

```json
{
  "file": "/usr/lib/x86_64-linux-gnu/libcuda.so.1",
  "func": "cuLaunchKernel",  ← Usually available
  "offset": 53
}
```

**Why func is available:**
- Shared libraries **must** export public API functions
- These symbols are in the dynamic symbol table (`.dynsym`)
- Required for dynamic linking
- Can't be stripped without breaking the library

### Your Binary (aes_test):

```json
{
  "file": "/workspace/.../bin/aes_test",
  "func": "",  ← Often empty for internal functions
  "offset": 102740
}
```

**Why func might be empty:**
- Internal/static functions may not be in dynamic symbol table
- Only exported symbols are guaranteed to be accessible
- Debug symbols (`.debug_info`) contain full information but tracer might not read them
- Inlined functions have no separate symbol

---

## How to Get More Information

### Option 1: Add Debug Symbols

Compile with `-g` flag:
```bash
gcc -g -o aes_test aes.c
# or
nvcc -g -G -o aes_test aes.cu
```

This adds DWARF debug information to the binary.

### Option 2: Don't Strip Symbols

Make sure not to run:
```bash
strip aes_test  # DON'T do this if you want symbols
```

### Option 3: Use addr2line

Even without function names in backtrace, you can manually resolve:

```bash
# Get source file and line number
addr2line -e aes_test -f 102740

# Example output:
# AES_cbc_128_encrypt
# /workspace/example/crypt-examples/libgpucrypto/src/aes.c:145
```

### Option 4: Disassemble

```bash
objdump -d aes_test > aes_test.asm
# Then search for offset 102740 in the assembly listing
```

---

## Summary

### Debug Symbols:
- **What**: Metadata mapping machine code to source code
- **Contains**: Function names, variable names, line numbers, type info
- **Size**: Makes binaries larger (2-10x)
- **Purpose**: Debugging, profiling, symbolication

### Offset in Backtrace:
- **What**: Byte offset from start of binary file
- **How**: `runtime_address - binary_base_address`
- **Purpose**: Locates exact instruction in the binary
- **Use**: Can disassemble or use addr2line to find source location

### Why `func` is Empty:
- Symbol not in dynamic symbol table
- Inlined or compiler-generated code
- Binary stripped (though yours says "not stripped")
- Tracer couldn't resolve the symbol

### Your Backtrace:
```
cuLaunchKernel (offset 53)          ← NVIDIA driver (has symbols)
  ↓
cudaLaunchKernel (offset 598)       ← CUDA runtime (has symbols)
  ↓
[unknown] (offset 102740)           ← Your code (no symbol found)
  ↓
[unknown] (offset 102959)           ← Your code (no symbol found)
  ↓
[GPU kernel executes]
```

Even without function names, the **offset values are enough** for you to use `addr2line` or `objdump` to find the exact location in your source code!
