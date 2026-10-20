#!/usr/bin/env python3
"""Stream the text column of one or more parquet files to stdout.

Documents are separated by U+001E so `ekushe-dict count` can track document frequency.
Usage: extract_text.py [--column text] FILE.parquet [...] | ekushe-dict count --out counts.tsv
"""
import sys
import pyarrow.parquet as pq

def main():
    args = sys.argv[1:]
    col = "text"
    if args[:1] == ["--column"]:
        col, args = args[1], args[2:]
    out = sys.stdout.buffer
    for path in args:
        pf = pq.ParquetFile(path)
        for batch in pf.iter_batches(columns=[col], batch_size=2000):
            for t in batch.column(0).to_pylist():
                if t:
                    out.write(t.encode("utf-8", "replace"))
                    out.write(b"\n\x1e")
    out.flush()

if __name__ == "__main__":
    main()
