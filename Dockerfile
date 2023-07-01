FROM rust:1.70-alpine as builder
RUN apk add alpine-sdk openssl-dev
WORKDIR /usr/src/channel-bot
COPY . .
RUN cargo install --path .

FROM alpine:3
RUN apk add openssl
COPY --from=builder /usr/local/cargo/bin/channel-bot /usr/local/bin/channel-bot
CMD ["channel-bot"]
