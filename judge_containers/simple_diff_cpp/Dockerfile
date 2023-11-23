FROM rust:latest as build
WORKDIR /usr/src/myapp
COPY simple_diff_judge .
RUN cargo build --release

FROM ubuntu:latest
RUN apt-get update && apt-get install -y g++
COPY --from=build /usr/src/myapp/target/release/judge /judge/bin/judge
COPY --from=build /usr/src/myapp/target/release/supervisor /judge/bin/supervisor
WORKDIR /judge
ENTRYPOINT [ "/judge/bin/judge" ]