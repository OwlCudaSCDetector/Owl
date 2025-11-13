#!/usr/bin/env python3
"""
OWL Results Visualization Script
Generates CDF plots for memory access patterns to visualize side-channel leaks
"""

import json
import argparse
import matplotlib.pyplot as plt
import numpy as np
from pathlib import Path
from collections import defaultdict, Counter

# Configuration: Which instruction operand to analyze
# Based on report.json - BB 1056 has the most leaking instructions
BB_ID = 1056
INSTR_ADDR = 3712  # 0xe80 - one of the top leaking instructions
OPERAND_IDX = 0    # Analyze first operand


def load_kernel_json(filepath):
    """Load kernel.json trace file"""
    with open(filepath, 'r') as f:
        return json.load(f)


def extract_memory_accesses(kernel_data):
    """
    Extract memory addresses converted to offsets within pools
    This matches how OWL analyzer processes the data
    """
    addresses = []
    
    for kernel in kernel_data['data']:
        # Get memory pools
        mem_pools = kernel.get('mp', [])
        
        graph = kernel.get('g', {})
        nodes = graph.get('nodes', [])
        
        for node in nodes:
            mem_accesses = node.get('mem_access', [])
            for mem_instr in mem_accesses:
                data_list = mem_instr.get('data', [])
                for data_item in data_list:
                    accesses = data_item.get('access', [])
                    for access in accesses:
                        memory_list = access.get('memory', [])
                        for mem in memory_list:
                            addr = mem.get('addr', 0)
                            count = mem.get('count', 1)
                            
                            # Convert absolute address to offset within pool
                            # (matching OWL's alloc.rs logic)
                            offset = None
                            for pool in mem_pools:
                                pool_start = pool['addr']
                                pool_size = pool['size']
                                if addr >= pool_start and addr < pool_start + pool_size:
                                    offset = addr - pool_start
                                    break
                            
                            # Only include in-pool addresses (T-table accesses)
                            if offset is not None:
                                # Add offset 'count' times
                                addresses.extend([offset] * count)
    
    return addresses


def compute_ecdf(data):
    """Compute empirical CDF"""
    if len(data) == 0:
        return np.array([]), np.array([])
    
    sorted_data = np.sort(data)
    n = len(sorted_data)
    y = np.arange(1, n + 1) / n
    return sorted_data, y


def extract_instruction_operand_accesses(kernel_data, bb_id, instr_addr, operand_idx=0):
    """
    Extract memory offsets for a specific instruction's operand.
    This matches OWL's methodology of testing individual operands.
    
    Note: The 'addr' values in the trace are ALREADY offsets within pools,
    not absolute addresses. OWL has already converted them during trace generation.
    """
    # Find the basic block
    for node in kernel_data['data'][0]['g']['nodes']:
        if node['id'] == bb_id:
            # Find the instruction
            for mem_instr in node['mem_access']:
                if mem_instr['addr'] == instr_addr:
                    # Get the specific operand
                    if operand_idx >= len(mem_instr['data']):
                        return []
                    
                    operand = mem_instr['data'][operand_idx]
                    offsets = []
                    
                    # Extract memory offsets
                    # Structure: operand['access'][i]['memory'] is a list of {addr, count} objects
                    # where 'addr' is already an offset (not absolute address)
                    for access_entry in operand.get('access', []):
                        memory_list = access_entry.get('memory', [])
                        
                        for mem_obj in memory_list:
                            offset = mem_obj['addr']  # Already an offset!
                            count = mem_obj.get('count', 1)
                            
                            # Add offset 'count' times (same access repeated)
                            offsets.extend([offset] * count)
                    
                    return offsets
    
    return []


def extract_all_instruction_accesses(kernel_data, bb_id, instr_addr):
    """
    Extract memory offsets for ALL operands of an instruction.
    This gives us more data points for the CDF.
    """
    # Find the basic block
    for node in kernel_data['data'][0]['g']['nodes']:
        if node['id'] == bb_id:
            # Find the instruction
            for mem_instr in node['mem_access']:
                if mem_instr['addr'] == instr_addr:
                    offsets = []
                    
                    # Collect offsets from ALL operands
                    for operand in mem_instr['data']:
                        for access_entry in operand.get('access', []):
                            memory_list = access_entry.get('memory', [])
                            
                            for mem_obj in memory_list:
                                offset = mem_obj['addr']
                                count = mem_obj.get('count', 1)
                                offsets.extend([offset] * count)
                    
                    return offsets
    
    return []


def plot_memory_access_cdf(fix_dirs, rnd_dirs, output_file='memory_cdf.png'):
    """
    Plot CDF of memory accesses for a specific leaking instruction
    Matches OWL's analysis methodology
    """
    # Define different line styles to distinguish overlapping lines
    line_styles = ['-', '--', '-.', ':', (0, (3, 1, 1, 1)), (0, (5, 2)), 
                   (0, (1, 1)), (0, (3, 5, 1, 5)), (0, (5, 1)), (0, (3, 1, 1, 1, 1, 1))]
    
    # Define different colors for better distinction
    colors = plt.cm.tab10(np.linspace(0, 1, 10))
    
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(16, 6))
    
    # Plot fixed-key traces
    print(f"\n Processing Fixed-Key Traces (BB:{BB_ID}, Instr:0x{INSTR_ADDR:x}, ALL operands)...")
    for i, trace_dir in enumerate(fix_dirs):
        kernel_file = Path(trace_dir) / 'kernel.json'
        if not kernel_file.exists():
            print(f"    Skipping {trace_dir} - kernel.json not found")
            continue
        
        print(f"   Loading trace {i}...")
        kernel_data = load_kernel_json(kernel_file)
        addresses = extract_all_instruction_accesses(kernel_data, BB_ID, INSTR_ADDR)
        
        if len(addresses) > 0:
            x, y = compute_ecdf(addresses)
            ax1.plot(x, y, alpha=0.9, label=f'Run {i}', linewidth=2.5,
                    linestyle=line_styles[i % len(line_styles)],
                    color=colors[i % len(colors)])
            print(f"    {len(addresses)} memory accesses found, {len(set(addresses))} unique offsets")
    
    ax1.set_title(f'Fixed Key - T-table Access CDF\n(BB {BB_ID}, Instr 0x{INSTR_ADDR:x})', 
                  fontsize=14, fontweight='bold')
    ax1.set_xlabel('Memory Offset (bytes)', fontsize=12)
    ax1.set_ylabel('Cumulative Probability', fontsize=12)
    ax1.grid(True, alpha=0.3)
    ax1.legend(fontsize=9, loc='best')
    
    # Plot random-key traces
    print(f"\n Processing Random-Key Traces (BB:{BB_ID}, Instr:0x{INSTR_ADDR:x}, ALL operands)...")
    for i, trace_dir in enumerate(rnd_dirs):
        kernel_file = Path(trace_dir) / 'kernel.json'
        if not kernel_file.exists():
            print(f"    Skipping {trace_dir} - kernel.json not found")
            continue
        
        print(f"   Loading trace {i}...")
        kernel_data = load_kernel_json(kernel_file)
        addresses = extract_all_instruction_accesses(kernel_data, BB_ID, INSTR_ADDR)
        
        if len(addresses) > 0:
            x, y = compute_ecdf(addresses)
            ax2.plot(x, y, alpha=0.9, label=f'Run {i}', linewidth=2.5,
                    linestyle=line_styles[i % len(line_styles)],
                    color=colors[i % len(colors)])
            print(f"    {len(addresses)} memory accesses found, {len(set(addresses))} unique offsets")
    
    ax2.set_title(f'Random Keys - T-table Access CDF\n(BB {BB_ID}, Instr 0x{INSTR_ADDR:x})', 
                  fontsize=14, fontweight='bold')
    ax2.set_xlabel('Memory Offset (bytes)', fontsize=12)
    ax2.set_ylabel('Cumulative Probability', fontsize=12)
    ax2.grid(True, alpha=0.3)
    ax2.legend(fontsize=9, loc='best')
    
    plt.tight_layout()
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"\n Plot saved to: {output_file}")
    plt.show()


def analyze_address_distribution(fix_dirs, rnd_dirs):
    """
    Analyze and compare memory access distributions between fixed and random keys.
    """
    print("\n" + "="*60)
    print("MEMORY ACCESS PATTERN ANALYSIS (Leaking Instruction)")
    print("="*60)
    print()
    print(f" Analyzing BB {BB_ID}, Instruction 0x{INSTR_ADDR:x}, ALL operands")
    print()
    
    # Load all offsets from fixed-key traces
    fix_offsets = []
    for trace_dir in fix_dirs:
        kernel_file = Path(trace_dir) / 'kernel.json'
        if kernel_file.exists():
            kernel_data = load_kernel_json(kernel_file)
            offsets = extract_all_instruction_accesses(kernel_data, BB_ID, INSTR_ADDR)
            fix_offsets.extend(offsets)
    
    # Load all offsets from random-key traces
    rnd_offsets = []
    for trace_dir in rnd_dirs:
        kernel_file = Path(trace_dir) / 'kernel.json'
        if kernel_file.exists():
            kernel_data = load_kernel_json(kernel_file)
            offsets = extract_all_instruction_accesses(kernel_data, BB_ID, INSTR_ADDR)
            rnd_offsets.extend(offsets)
    
    # Check if we have data
    if len(fix_offsets) == 0 and len(rnd_offsets) == 0:
        print(" No memory accesses found!")
        print("   Check that BB_ID, INSTR_ADDR, and OPERAND_IDX are correct.")
        print("="*60)
        return
    
    # Statistics for fixed key
    print(" Fixed Key Statistics:")
    if len(fix_offsets) > 0:
        unique_fix = len(set(fix_offsets))
        print(f"  Total accesses: {len(fix_offsets)}")
        print(f"  Unique offsets: {unique_fix}")
        print(f"  Min offset: 0x{min(fix_offsets):x}")
        print(f"  Max offset: 0x{max(fix_offsets):x}")
        
        # Top 10 most accessed offsets
        counter = Counter(fix_offsets)
        print(f"  Top 10 most accessed offsets:")
        for offset, count in counter.most_common(10):
            print(f"    0x{offset:x}: {count} times")
    else:
        unique_fix = 0
        print("  No accesses found")
    
    print()
    
    # Statistics for random keys
    print(" Random Key Statistics:")
    if len(rnd_offsets) > 0:
        unique_rnd = len(set(rnd_offsets))
        print(f"  Total accesses: {len(rnd_offsets)}")
        print(f"  Unique offsets: {unique_rnd}")
        print(f"  Min offset: 0x{min(rnd_offsets):x}")
        print(f"  Max offset: 0x{max(rnd_offsets):x}")
        
        # Top 10 most accessed offsets
        counter = Counter(rnd_offsets)
        print(f"  Top 10 most accessed offsets:")
        for offset, count in counter.most_common(10):
            print(f"    0x{offset:x}: {count} times")
    else:
        unique_rnd = 0
        print("  No accesses found")
    
    print()
    print(" Interpretation:")
    if len(fix_offsets) > 0 and len(rnd_offsets) > 0:
        if unique_fix < unique_rnd:
            print("    Fixed key has FEWER unique offsets → Suggests leakage")
            print("    Fixed key accesses same locations repeatedly")
            print("    Random keys spread across more memory locations")
        elif unique_fix > unique_rnd:
            print("    Fixed key has MORE unique offsets → Unexpected")
        else:
            print("    Same number of unique offsets → No clear pattern")
    print("="*60)


def main():
    parser = argparse.ArgumentParser(description='Visualize OWL analysis results')
    parser.add_argument('--results-dir', type=str, 
                       default='example/crypt-examples/libgpucrypto/owl_results/0',
                       help='Path to owl_results directory')
    parser.add_argument('--output', type=str, default='memory_cdf.png',
                       help='Output filename for plot')
    parser.add_argument('--runs', type=int, default=10,
                       help='Number of runs to analyze')
    
    args = parser.parse_args()
    
    results_path = Path(args.results_dir)
    
    # Build paths to trace directories
    fix_dirs = [results_path / 'fix' / str(i) for i in range(args.runs)]
    rnd_dirs = [results_path / 'rnd' / str(i) for i in range(args.runs)]
    
    # Filter existing directories
    fix_dirs = [d for d in fix_dirs if d.exists()]
    rnd_dirs = [d for d in rnd_dirs if d.exists()]
    
    print(f"\n Found {len(fix_dirs)} fixed-key traces")
    print(f" Found {len(rnd_dirs)} random-key traces")
    
    if len(fix_dirs) == 0 or len(rnd_dirs) == 0:
        print("\n Error: No trace files found!")
        print(f"   Looking in: {results_path}")
        print("\n Make sure you've run the AES test first:")
        print("   cd example/crypt-examples/libgpucrypto")
        print("   ./run_aes.sh")
        return
    
    # Analyze distributions
    analyze_address_distribution(fix_dirs, rnd_dirs)
    
    # Plot CDFs
    plot_memory_access_cdf(fix_dirs, rnd_dirs, args.output)


if __name__ == '__main__':
    main()
