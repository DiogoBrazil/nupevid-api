FROM rust:1.90.0 as builder
WORKDIR /app

ENV SQLX_OFFLINE=true

COPY . .
RUN cargo build --release

FROM debian:stable-slim
WORKDIR /usr/local/bin

# Instalar dependências necessárias
RUN apt-get update && apt-get install -y \
    libpq-dev \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copia o binário compilado do estágio de build
COPY --from=builder /app/swagger.yml .
COPY --from=builder /app/target/release/nupevid-api .

COPY --from=builder /app/.env .

RUN chmod +x ./nupevid-api

CMD ["./nupevid-api"]
