
import os
import json
import tarfile

def index_tarball(path):
    outfile = os.path.join("output", os.path.basename(path) + ".jsonl")
    with tarfile.open(path) as tarball:
        with open(outfile, "w") as output:
            for member in tarball:
                row = json.dumps({
                    "archive": path,
                    "filename": member.name,
                    "size": member.size
                })
                output.write(row)
                output.write("\n")

if __name__ == "__main__":
    import sys
    for ln in sys.stdin:
        index_tarball(ln.strip())
