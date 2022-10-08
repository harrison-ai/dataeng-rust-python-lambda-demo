import sys
import json
import tarfile
from io import BytesIO

import boto3


def index_tarball(client, input_bucket, input_key, output_bucket, output_prefix):
    input = client.get_object(Bucket=input_bucket, Key=input_key)["Body"]

    output = BytesIO()
    with tarfile.open(fileobj=input, mode="r|") as tarball:
        for member in tarball:
            row = json.dumps(
                {"archive": input_key, "filename": member.name, "size": member.size}
            )
            output.write(row.encode("utf-8"))
            output.write(b"\n")

    output.seek(0)
    archive_name = input_key.rsplit("/", 1)[-1]
    client.put_object(
        Bucket=output_bucket,
        Key=f"{output_prefix}/{archive_name}.jsonl",
        Body=output,
    )


if __name__ == "__main__":
    client = boto3.client("s3")
    for ln in sys.stdin:
        index_tarball(
            client,
            "rfkelly-rust-python-lambda-demo",
            ln.strip(),
            "rfkelly-rust-python-lambda-demo",
            "output",
        )
