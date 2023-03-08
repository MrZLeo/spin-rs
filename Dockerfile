FROM rust:1.67.1-alpine3.16

COPY . /
RUN cargo install --path .
CMD ["spin-rs"]
