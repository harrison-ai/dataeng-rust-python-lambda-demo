
import os
import json
import tarfile

def index_tarball(input_path, output_path):
    with tarfile.open(input_path) as tarball:
        with open(output_path, "w") as output:
            for member in tarball:
                row = json.dumps({
                    "archive": input_path,
                    "filename": member.name,
                    "size": member.size
                })
                output.write(row)
                output.write("\n")

if __name__ == "__main__":
    for nm in os.listdir("input"):
        input_path = os.path.join("input", nm)
        output_path = os.path.join("output", os.path.basename(input_path) + ".jsonl")
        index_tarball(input_path, output_path)
