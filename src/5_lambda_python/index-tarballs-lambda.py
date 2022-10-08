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
    archive_name = input_key.rsplit("/", 1)[-1]
    client.put_object(
        Bucket=output_bucket,
        Key=f"{output_prefix}/{archive_name}.jsonl",
        Body=output,
    )

CLIENT = boto3.client("s3")

def lambda_handler(event, _context):
    for record in event["Records"]:
        index_tarball(
            CLIENT,
            "rfkelly-rust-python-lambda-demo",
            json.loads(record["body"]),
            "rfkelly-rust-python-lambda-demo",
            "output",
        )
