FROM rust:latest AS builder
WORKDIR /build
COPY . .
RUN [ "cargo", "build", "-r" ]

FROM debian:11-slim
WORKDIR /app
COPY ./img ./
COPY --from=builder /build/target/release/rpgbot ./
CMD [ "./rpgbot" ]