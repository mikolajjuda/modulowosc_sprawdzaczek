FROM rust:latest as build
RUN apt-get update && apt-get install -y libseccomp-dev
WORKDIR /usr/src/myapp
COPY simple_diff_judge .
RUN cargo build --release

FROM ubuntu:latest
RUN apt-get update && apt-get install -y g++ libseccomp-dev
COPY --from=build /usr/src/myapp/target/release/judge /judge/bin/judge
COPY --from=build /usr/src/myapp/target/release/supervisor /judge/bin/supervisor
WORKDIR /judge
ENTRYPOINT [ "/judge/bin/judge" ]