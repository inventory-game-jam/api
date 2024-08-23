FROM rust:1.80.1
WORKDIR /usr/src/api
COPY . .
RUN cargo build --release
EXPOSE 3000

CMD ["./target/release/api"]
