FROM rust:1.68.2

COPY . /
RUN cargo install --path .
RUN apt-get update && apt-get install -y python3-pip
RUN pip3 install flask

CMD ["spin-rs"]
