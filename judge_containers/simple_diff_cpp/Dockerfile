FROM rust:latest as build
WORKDIR /usr/src/myapp
COPY simple_diff_judge .
RUN cargo build --release

FROM alpine:latest
RUN apk add --no-cache g++ gcompat
COPY --from=build /usr/src/myapp/target/release/judge /judge/bin/judge
COPY --from=build /usr/src/myapp/target/release/runner /judge/bin/runner
WORKDIR /judge
ENTRYPOINT [ "/judge/bin/judge" ]