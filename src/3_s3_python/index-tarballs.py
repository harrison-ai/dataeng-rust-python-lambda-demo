import sys
import json
import tarfile
from io import BytesIO

import boto3


def index_tarball(s3client, bucket, input_key, output_key):
    input = s3client.get_object(Bucket=bucket, Key=input_key)["Body"]

    output = BytesIO()
    with tarfile.open(fileobj=input, mode="r|") as tarball:
        for member in tarball:
            row = json.dumps(
                {"archive": input_key, "filename": member.name, "size": member.size}
            )
            output.write(row.encode("utf-8"))
            output.write(b"\n")

    output.seek(0)
    s3client.put_object(
        Bucket=bucket,
        Key=output_key,
        Body=output,
    )


if __name__ == "__main__":
    client = boto3.client("s3")
    for ln in sys.stdin:
        input_key = ln.strip()
        archive_name = input_key.rsplit("/", 1)[-1]
        output_key = f"output/py/{archive_name}.jsonl"
        index_tarball(client, "rfkelly-rust-python-lambda-demo", input_key, output_key)
