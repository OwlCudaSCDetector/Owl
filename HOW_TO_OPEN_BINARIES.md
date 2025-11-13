# How to Open Binary Files

## In VS Code (Graphical)

### Method 1: Hex Editor Extension (Best for Browsing)

**Install the official Microsoft Hex Editor:**
1. Open Extensions (Ctrl+Shift+X)
2. Search for "Hex Editor" by Microsoft (`ms-vscode.hexeditor`)
3. Click Install

**Usage:**
- Right-click any binary file in Explorer
- Select "Open With..." → "Hex Editor"
- View/edit bytes in hex format with ASCII sidebar

**Features:**
- ✅ Visual hex/ASCII representation
- ✅ Search for hex patterns
- ✅ Edit bytes directly
- ✅ Data inspector (view as int, float, etc.)
- ✅ Large file support

### Method 2: Binary Viewer Extension

Alternative: `qiaojie.binary-viewer` - simpler hex viewer

---

## In Terminal (Immediate, No Installation)

### 1. **Check Binary Type**
```bash
file <binary-file>
```
**Example:**
```bash
$ file bin/aes_test
bin/aes_test: ELF 64-bit LSB pie executable, x86-64, 
              with debug_info, not stripped
```

### 2. **View Function Names (Symbols)**
```bash
nm <binary-file> | less
```
**Filter for functions only:**
```bash
nm <binary-file> | grep -E " [Tt] "
```
**Example:**
```bash
$ nm bin/aes_test | grep AES
000000000001a710 T AES_cbc_encrypt
0000000000019be0 T AES_encrypt
0000000000019740 T AES_set_encrypt_key
```

### 3. **Disassemble (View Assembly Code)**
```bash
objdump -d <binary-file> | less
```
**Specific function:**
```bash
objdump -d <binary-file> | grep -A50 "<function_name>"
```
**Example:**
```bash
$ objdump -d bin/aes_test | grep -A20 "AES_encrypt>:"
```

### 4. **View Hex Dump**
```bash
hexdump -C <binary-file> | less
```
**First 512 bytes:**
```bash
hexdump -C <binary-file> | head -32
```
**Specific offset:**
```bash
hexdump -C -s 102740 -n 256 <binary-file>
# -s: skip to offset 102740
# -n: read 256 bytes
```

### 5. **Extract Readable Strings**
```bash
strings <binary-file> | less
```
**Find specific text:**
```bash
strings <binary-file> | grep -i "aes"
```

### 6. **View ELF Header**
```bash
readelf -h <binary-file>
```
**Example output:**
```
ELF Header:
  Magic:   7f 45 4c 46 02 01 01 00
  Class:                             ELF64
  Data:                              2's complement, little endian
  Version:                           1 (current)
  OS/ABI:                            UNIX - System V
  Machine:                           Advanced Micro Devices X86-64
  Entry point address:               0x13a60
```

### 7. **View All Sections**
```bash
readelf -S <binary-file>
```
**Shows:**
- `.text` - executable code
- `.data` - initialized data
- `.rodata` - read-only data (strings, constants)
- `.bss` - uninitialized data
- `.symtab` - symbol table
- `.debug_info` - debug symbols

### 8. **Find Source Line for Offset**
```bash
addr2line -e <binary-file> -f <offset>
```
**Example:**
```bash
$ addr2line -e bin/aes_test -f 102740
AES_cbc_128_encrypt_gpu
/workspace/.../aes.c:145
```

### 9. **Demangle C++ Names**
```bash
nm <binary-file> | c++filt
```
**Or specific name:**
```bash
echo "_Z23AES_cbc_128_encrypt_gpu..." | c++filt
```

### 10. **Open in Debugger**
```bash
gdb <binary-file>
```
**Inside GDB:**
```gdb
(gdb) info functions           # List all functions
(gdb) disassemble main         # Disassemble function
(gdb) x/32xb 0x191d4          # Examine 32 bytes at address
(gdb) info symbol 0x191d4     # What symbol is at address?
```

---

## Practical Examples for Your AES Binary

### Find Offset 102740 in Hex Editor
```bash
# Convert decimal to hex
echo "obase=16; 102740" | bc
# Output: 191D4

# Then in hex editor, go to offset 0x191D4
```

### View Assembly at Specific Offset
```bash
objdump -d bin/aes_test | grep -B5 -A10 "191d4:"
```

### Find Which Function Contains Offset
```bash
# Get all function addresses
nm bin/aes_test | grep " T " | sort

# Or use objdump
objdump -d bin/aes_test | grep "^[0-9a-f]* <" | grep "00019"
```

### Extract Just Code Section
```bash
objdump -d -j .text bin/aes_test > aes_test_code.asm
```

### Compare Two Binaries
```bash
# Method 1: Byte-by-byte
cmp -l binary1 binary2

# Method 2: Hex diff
hexdump -C binary1 > bin1.hex
hexdump -C binary2 > bin2.hex
diff bin1.hex bin2.hex
```

### Find All CUDA-related Symbols
```bash
nm bin/aes_test | grep -i cuda
nm bin/aes_test | grep -i kernel
```

---

## Advanced: Navigating to Specific Offsets

### Example: Going to Offset 102740

**In Hex Editor (VS Code):**
1. Open binary with Hex Editor
2. Press `Ctrl+G` (Go to Offset)
3. Type `102740` (decimal) or `0x191D4` (hex)
4. Press Enter

**In hexdump:**
```bash
hexdump -C -s 102740 -n 128 bin/aes_test
```

**In objdump:**
```bash
objdump -d bin/aes_test | grep "191d4:"
```

**In GDB:**
```gdb
(gdb) x/32xb 0x191d4
(gdb) disassemble 0x191d4,+64
```

---

## Understanding What You See

### ELF Magic Number (First 4 bytes)
```
7f 45 4c 46  →  .ELF
```
Every ELF binary starts with this!

### Function Prologue (Common Assembly Pattern)
```assembly
push   %rbp            # 55
mov    %rsp,%rbp       # 48 89 e5
sub    $0x20,%rsp      # 48 83 ec 20
```

### Function Epilogue
```assembly
leave                  # c9
ret                    # c3
```

### CUDA Kernel Launch Pattern
Look for calls to `cudaLaunchKernel`:
```assembly
call   18ff0 <cudaLaunchKernel@plt>
```

---

## Quick Reference Table

| Task | Command |
|------|---------|
| **Binary type** | `file <binary>` |
| **Function list** | `nm <binary> \| grep " T "` |
| **Disassemble** | `objdump -d <binary>` |
| **Hex view** | `hexdump -C <binary>` |
| **Strings** | `strings <binary>` |
| **Sections** | `readelf -S <binary>` |
| **Symbols** | `readelf -s <binary>` |
| **Headers** | `readelf -h <binary>` |
| **Find line** | `addr2line -e <binary> <offset>` |
| **Demangle** | `c++filt <mangled-name>` |
| **Debug** | `gdb <binary>` |
| **Hex editor** | Right-click → Open With → Hex Editor |

---

## Tips

### 1. Pipe to `less` for Large Output
```bash
objdump -d bin/aes_test | less
```
Navigate with:
- `Space` - next page
- `b` - previous page
- `/pattern` - search forward
- `?pattern` - search backward
- `q` - quit

### 2. Save Output to File
```bash
objdump -d bin/aes_test > disassembly.asm
nm bin/aes_test > symbols.txt
hexdump -C bin/aes_test > hexdump.txt
```

### 3. Combine Commands
```bash
# Find AES functions and show their disassembly
nm bin/aes_test | grep AES | while read addr type name; do
    echo "=== $name ==="
    objdump -d bin/aes_test | grep -A30 "$name>:"
done
```

### 4. Watch for These Patterns

**In kernel.json offset → Binary:**
```json
"offset": 102740  →  Hex: 0x191D4  →  Find in binary at byte 102740
```

**Function boundaries:**
```assembly
0000000000019190 <function_name>:    ← Function starts here
   19190:   push   %rbp
   ...
   191d4:   movabs $0x1,%rdx        ← Your offset 102740!
   ...
   19250:   ret                      ← Function ends here
```

---

## Summary

**For browsing:** Install Hex Editor extension (best visual experience)
**For analysis:** Use terminal tools (more powerful, scriptable)
**For debugging:** Use GDB (interactive, can set breakpoints)
**For offset lookup:** Use objdump + grep (fastest way to find specific bytes)

All these methods give you different views of the same binary data!
