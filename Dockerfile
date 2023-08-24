FROM rust:latest as build
RUN USER=root cargo new wildberries_test
WORKDIR /wildberries_test
RUN echo $(pwd)
RUN echo $(ls)
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs
COPY ./src ./src
RUN rm ./target/release/wildberries_test*
RUN cargo build --release

FROM rust:latest
COPY --from=build /wildberries_test/target/release/wildberries_test .
CMD ["./wildberries_test"]
