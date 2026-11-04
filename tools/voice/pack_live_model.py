#!/usr/bin/env python3
"""Pack the live speech model's files into one .ekm download (the format ekushe-voice unpacks).

  pack_live_model.py OUT.ekm encoder.int8.onnx decoder.int8.onnx joiner.int8.onnx tokens.txt CREDITS.txt

Format: b"EKZM1\\n", the number of files (u32 LE), then for each file its name length (u16 LE), name
(UTF-8) and size (u64 LE), then the files' bytes in the same order.
"""
import os, struct, sys

out, files = sys.argv[1], sys.argv[2:]
with open(out, "wb") as f:
    f.write(b"EKZM1\n" + struct.pack("<I", len(files)))
    for p in files:
        name = os.path.basename(p).encode()
        f.write(struct.pack("<H", len(name)) + name + struct.pack("<Q", os.path.getsize(p)))
    for p in files:
        with open(p, "rb") as src:
            while chunk := src.read(1 << 20):
                f.write(chunk)
print(out, os.path.getsize(out), "bytes")
