import os
import json
import tarfile
import boto3


def index_tarball(client, input_bucket, input_key, output_bucket, output_prefix):
    output_key = f"{output_prefix}/partition={input_key[:2]}/input_key.jsonl"
    outfile = os.path.join("output", os.path.basename(input_key) + ".jsonl")
    input = client.get_object(Bucket=input_bucket, Key=input_key)["Body"]
    with tarfile.open(fileobj=input, mode="r|") as tarball:
        with open(outfile, "w") as output:
            for member in tarball:
                row = json.dumps({"filename": member.name, "size": member.size})
                output.write(row)
                output.write("\n")


if __name__ == "__main__":
    import sys
    import boto3

    client = boto3.client("s3")
    for ln in sys.stdin:
        index_tarball(
            client,
            "rfkelly-rust-python-lambda-demo",
            ln.strip(),
            "rfkelly-rust-python-lambda-demo",
            "output",
        )
