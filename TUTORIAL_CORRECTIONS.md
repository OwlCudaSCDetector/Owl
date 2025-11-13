# AES_ANALYSIS_TUTORIAL.md Corrections Summary

This document summarizes the corrections made to `AES_ANALYSIS_TUTORIAL.md` to ensure accuracy with the actual Owl implementation.

## Major Corrections

### 1. Kernel Signature and Parameters

**Previous (Incorrect):**
```cuda
__global__ void AES_cbc_128_encrypt_kernel_SharedMem(
    const u8* in,
    u8* out,
    const u32* rkey,
    const u8* pIv,
    u8* pIvOut,
    u32 nBlock,
    u8* Te
)
```

**Corrected:**
```cuda
__global__ void AES_cbc_128_encrypt_kernel_SharedMem(
    const uint8_t       *in_all,      // All flows' input
    uint8_t             *out_all,     // All flows' output
    const uint32_t      *pkt_offset,  // Flow boundaries
    const uint8_t       *keys,        // Per-flow keys
    uint8_t             *ivs,         // Per-flow IVs
    const unsigned int  num_flows,    // Number of flows
    uint8_t             *checkbits    // Optional validation
)
```

**Key Differences:**
- The kernel processes **flows** (variable-length packets), not individual blocks
- Uses `pkt_offset` array to define flow boundaries: flow `i` has data from `in_all[pkt_offset[i]]` to `in_all[pkt_offset[i+1]]`
- Each thread processes an entire flow (multiple blocks), not one block per thread
- Separate keys and IVs for each flow, not one global set

### 2. Processing Model

**Previous (Incorrect):**
- One thread per 16-byte block
- Pre-computed round keys passed as parameter
- Direct thread-to-block mapping

**Corrected:**
- One thread per **flow** (which contains multiple blocks)
- On-the-fly key expansion using `next_rk_128()` function
- CBC loop within each thread processing all blocks in the flow

### 3. T-box Storage and Loading

**Previous (Incorrect):**
```cuda
u8* Te  // T-box tables in device memory (4KB)
sharedMemory[0][i] = ((u32*)Te)[i];  // Copy from device memory
```

**Corrected:**
```cuda
// T-boxes are in CONSTANT memory, not passed as parameter
__constant__ __device__ uint32_t Te0_ConstMem[256];
__constant__ __device__ uint32_t Te1_ConstMem[256];
// ... etc

// Copied to shared memory from constant memory
__shared__ uint32_t shared_Te0[256];
shared_Te0[i] = Te0_ConstMem[i];
```

**Key Difference:** 
- T-boxes stored in constant memory (`Te0_ConstMem`, etc.), not passed as parameter
- Also loads `shared_Rcon[10]` for round constants

### 4. AES Encryption Implementation

**Previous (Incorrect):**
- Inline T-box lookups in kernel
- Pre-expanded round keys

**Corrected:**
- Calls `AES_128_encrypt()` device function from `aes_core.h`
- On-the-fly key expansion using `next_rk_128()`
- Round keys maintained in local `rk[4]` array

### 5. Fixed Key Generation

**Previous (Incorrect):**
```c
const u8 fixed_key[16] = {
    0x2b, 0x7e, 0x15, 0x16, ...  // Hardcoded hex
};
```

**Corrected:**
```c
const char* fix_keys = "ThisIsAFixKey";
unsigned char keys[20];
strcpy((char*)keys, fix_keys);
set_fix((*i).key, key_bits / 8, keys);
```

**Key Difference:** The fixed key is derived from the string "ThisIsAFixKey", not a hardcoded hex array.

### 6. CBC Processing

**Previous (Incorrect):**
```cuda
// Single block processing
if (blockId == 0) {
    s0 ^= IV[0];
} else {
    s0 ^= previous_ciphertext[0];
}
```

**Corrected:**
```cuda
// Loop through all blocks in the flow
while (len >= AES_BLOCK_SIZE) {
    // XOR with IV or previous ciphertext (64-bit operations)
    ((uint64_t*)out)[0] = ((uint64_t*)in)[0] ^ ((uint64_t*)iv)[0];
    ((uint64_t*)out)[1] = ((uint64_t*)in)[1] ^ ((uint64_t*)iv)[1];
    
    // Encrypt the block
    AES_128_encrypt(out, out, key, 
                    shared_Te0, shared_Te1, shared_Te2, shared_Te3,
                    shared_Rcon);
    
    // Update IV for next block
    iv = out;
    
    // Move to next block
    in += AES_BLOCK_SIZE;
    out += AES_BLOCK_SIZE;
    len -= AES_BLOCK_SIZE;
}
```

**Key Differences:**
- Each thread loops through multiple blocks in its assigned flow
- Uses 64-bit XOR operations for efficiency
- Handles partial blocks with padding

### 7. Variable Naming

**Incorrect:** `u8`, `u32`, `sharedMemory[][]`

**Corrected:** `uint8_t`, `uint32_t`, `shared_Te0[]`, `shared_Te1[]`, etc.

## Sections Updated

1. **GPU AES Implementation in libgpucrypto**
   - Corrected kernel signature
   - Updated processing model description
   - Fixed T-box loading mechanism
   - Added accurate implementation details

2. **Kernel Implementation**
   - Complete rewrite with actual code flow
   - Added `AES_128_encrypt()` function details
   - Added on-the-fly key expansion explanation
   - Corrected CBC loop implementation

3. **The Side-Channel Vulnerability**
   - Updated to reference actual implementation
   - Corrected code examples
   - Added three leakage points explanation

4. **Test Binary Description**
   - Fixed command-line usage
   - Corrected fixed key generation mechanism
   - Updated expected outputs

5. **How Owl Detects the Leak**
   - Updated memory access pattern examples
   - Corrected variable names in examples

## Files Examined for Accuracy

- `/home/alan/Documents/Owl/example/crypt-examples/libgpucrypto/aes_kernel.cu` - Main kernel implementation
- `/home/alan/Documents/Owl/example/crypt-examples/libgpucrypto/aes_core.h` - AES core functions and constants
- `/home/alan/Documents/Owl/example/crypt-examples/libgpucrypto/test/aes_test.cc` - Test program and key generation

## Verification

All corrections have been verified against the actual source code in the Owl repository. The tutorial now accurately reflects:

1. The actual kernel signature and parameter structure
2. The flow-based processing model
3. The constant memory T-box storage
4. The on-the-fly key expansion mechanism
5. The actual test program behavior and options
6. The real side-channel vulnerability locations

## Impact

These corrections ensure that users following the tutorial will:
- Understand the actual implementation, not pseudocode
- See accurate code examples matching the real source
- Know the correct parameters and data structures
- Successfully run and analyze the AES example
- Understand where the actual vulnerability exists in the code
