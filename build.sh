cd judge_containers/simple_diff_cpp
docker build -t simple_diff_cpp .
cd ../custom_loop_python
docker build -t custom_loop_python .
cd ../..
docker run --rm -v ./judge_manager:/usr/src/myapp -w /usr/src/myapp golang go build -v