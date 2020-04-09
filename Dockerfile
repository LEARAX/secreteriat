FROM rust:1.42

WORKDIR /usr/src/secreteriat
COPY . .

RUN cargo install --path .

CMD ["secreteriat"]
