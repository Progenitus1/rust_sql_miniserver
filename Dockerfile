FROM rust:latest

# Copy apps and libs
COPY ./common ./common
COPY ./persistence ./persistence
COPY ./query_parser ./query_parser
COPY ./sql_server ./sql_server
COPY ./transaction_control ./transaction_control

# Copy config files
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY ./.env ./.env

RUN cargo build --release
EXPOSE 9000

CMD ["./target/release/sql_server"]
