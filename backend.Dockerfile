# -alpine3.14 ->
# error: linking with `cc` failed: exit status: 1
# ...
# /usr/lib/gcc/x86_64-alpine-linux-musl/10.3.1/../../../../x86_64-alpine-linux-musl/bin/ld: cannot find crti.o: No such file or directory
# perhaps install glibc so... avoid alpine for now
FROM rust:1.55 

# copy the library code (separate rust project)
COPY ./pp_lib/Cargo.toml /opt/pp_lib/
COPY ./pp_lib/Cargo.lock /opt/pp_lib/
# copy the source code for the private dependency
COPY ./pp_lib/src/ /opt/pp_lib/src/

# copy the backend API code
COPY ./pp_backend_api/Cargo.toml /opt/pp_backend_api/
COPY ./pp_backend_api/Cargo.lock /opt/pp_backend_api/
WORKDIR /opt/pp_backend_api
# dummy build to get the dependencies cached
RUN mkdir -p /opt/pp_backend_api/src && \
    touch /opt/pp_backend_api/src/lib.rs && \
    cargo build --release

# copy the source code for the bin project
COPY ./pp_backend_api/src/ /opt/pp_backend_api/src/
# this time build for real
RUN cargo build --release

# not this: ARG DOCKER_DB_HOST
ENV RUST_BACKTRACE=1
EXPOSE 3000
CMD ["/opt/pp_backend_api/target/release/pp_backend_api"]
