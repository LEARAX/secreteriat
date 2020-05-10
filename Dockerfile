FROM rust:1.43.1

WORKDIR /usr/src/secreteriat
COPY . .

RUN cargo install --path .

CMD ["secreteriat"]
