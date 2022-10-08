import sys
import json
import tarfile
from io import BytesIO

import boto3


def index_tarball(client, input_bucket, input_key, output_bucket, output_prefix):
    input = client.get_object(
        Bucket=input_bucket,
        Key=input_key
    )["Body"]

    output = BytesIO()
    with tarfile.open(fileobj=input, mode="r|") as tarball:
        for member in tarball:
            row = json.dumps({"filename": member.name, "size": member.size})
            output.write(row.encode("utf-8"))
            output.write(b"\n")

    output.seek(0)
    client.put_object(
        Bucket=output_bucket,
        # FIXME: need basename
        Key=f"{output_prefix}/partition={input_key[:2]}/{input_key}.jsonl",
        Body=output,
    )

def handler(event, _context):
    client = boto3.client("s3")
    for record in event["Records"]:
        index_tarball(
            client,
            "rfkelly-rust-python-lambda-demo",
            record["body"],
            "rfkelly-rust-python-lambda-demo",
            "output",
        )
