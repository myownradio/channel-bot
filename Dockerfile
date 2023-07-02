FROM rust:1.70 as builder
WORKDIR /usr/src/channel-bot
COPY . .
RUN cargo install --path .

FROM ubuntu:20.04
RUN apt-get update && \
    apt-get install -y libssl1.1 ca-certificates && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/channel-bot /channel-bot
CMD ["/channel-bot"]
