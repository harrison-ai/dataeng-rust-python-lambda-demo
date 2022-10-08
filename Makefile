
AWS_PROFILE=harrison-sandpit

publish:
	docker build -f ./src/6_lambda_rust/Dockerfile-x86_64 -t 929097612753.dkr.ecr.ap-southeast-2.amazonaws.com/index-tarballs-rust-x86_64:latest .
	docker build -f ./src/6_lambda_rust/Dockerfile-arm64 -t 929097612753.dkr.ecr.ap-southeast-2.amazonaws.com/index-tarballs-rust-arm64:latest .
	docker push 929097612753.dkr.ecr.ap-southeast-2.amazonaws.com/index-tarballs-rust-x86_64
	docker push 929097612753.dkr.ecr.ap-southeast-2.amazonaws.com/index-tarballs-rust-arm64
	aws lambda update-function-code --function-name index-tarballs-rust-x86-64 --architectures x86_64 --image-uri 929097612753.dkr.ecr.ap-southeast-2.amazonaws.com/index-tarballs-rust-x86_64:latest
	aws lambda update-function-code --function-name index-tarballs-rust-arm64 --architectures arm64 --image-uri 929097612753.dkr.ecr.ap-southeast-2.amazonaws.com/index-tarballs-rust-arm64:latest

	mkdir -p ./target/python/ && rm -f ./target/python/index-tarballs.zip
	zip -j ./target/python/index-tarballs.zip ./src/5_lambda_python/lambda_function.py
	aws lambda update-function-code --function-name index-tarballs-python-x86_64 --architectures x86_64 --zip-file fileb://target/python/index-tarballs.zip
	aws lambda update-function-code --function-name index-tarballs-python-arm64 --architectures arm64 --zip-file fileb://target/python/index-tarballs.zip


enqueue:
	for QUEUE in python-arm64 python-x86_64 rust-arm64 rust-x86_64; do aws sqs send-message-batch --queue-url https://sqs.ap-southeast-2.amazonaws.com/929097612753/index-tarballs-$$QUEUE --entries file://enqueue.json --output text; done;
	
purge:
	for QUEUE in python-arm64 python-x86_64 rust-arm64 rust-x86_64; do aws sqs purge-queue --queue-url https://sqs.ap-southeast-2.amazonaws.com/929097612753/index-tarballs-$$QUEUE; done;
