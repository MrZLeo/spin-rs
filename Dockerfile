FROM rust:1.68.2

COPY . /
RUN cargo install --path .
CMD ["spin-rs"]
