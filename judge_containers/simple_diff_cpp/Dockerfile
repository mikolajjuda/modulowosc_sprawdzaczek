FROM rust:latest as build
WORKDIR /usr/src/myapp
COPY simple_diff_judge .
ENV RUSTFLAGS='-C target-feature=+crt-static'
RUN cargo build --release --target x86_64-unknown-linux-gnu

FROM ubuntu:latest
RUN apt-get update && apt-get install -y g++
COPY --from=build /usr/src/myapp/target/x86_64-unknown-linux-gnu/release/judge /judge/bin/judge
COPY --from=build /usr/src/myapp/target/x86_64-unknown-linux-gnu/release/supervisor /judge/bin/supervisor
WORKDIR /judge
ENTRYPOINT [ "/judge/bin/judge" ]