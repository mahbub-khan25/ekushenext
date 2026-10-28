#!/usr/bin/env python3
"""Stream the text column of one or more parquet files to stdout.

Documents are separated by U+001E so `ekushe-dict count` can track document frequency.
Usage: extract_text.py [--column text] FILE.parquet [...] | ekushe-dict count --out counts.tsv
"""
import sys
import pyarrow as pa
import pyarrow.parquet as pq

# The default allocator keeps freed memory, and over a 5 GB file the process grew past 4.8 GB:
# use the system allocator and release memory after every row group
pa.set_memory_pool(pa.system_memory_pool())

def main():
    args = sys.argv[1:]
    col = "text"
    if args[:1] == ["--column"]:
        col, args = args[1], args[2:]
    out = sys.stdout.buffer
    for path in args:
        pf = pq.ParquetFile(path, pre_buffer=False)
        for i in range(pf.num_row_groups):
            table = pf.read_row_group(i, columns=[col])
            for t in table.column(0).to_pylist():
                if t:
                    out.write(t.encode("utf-8", "replace"))
                    out.write(b"\n\x1e")
            del table
            pa.default_memory_pool().release_unused()
    out.flush()

if __name__ == "__main__":
    main()
