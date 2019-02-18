# Use the nightly slim image for the build image, which we will not
# use for the final product.
FROM rustlang/rust:nightly-slim AS build

RUN apt-get update && apt-get install -qy libssl-dev pkg-config

WORKDIR /srv/app

ADD "Cargo.toml" "Cargo.lock" "./"
ADD "src" "./src/"

RUN ls -lR .; cargo install --path .

# Use a basic, slim base image
FROM debian:9.7-slim

RUN apt-get update && apt-get install -qy libssl-dev pkg-config

COPY --from=build /usr/local/cargo/bin/krye_io /usr/local/bin/krye_io

WORKDIR /srv/app

ADD "public" "./public"

CMD /usr/local/bin/krye_io
