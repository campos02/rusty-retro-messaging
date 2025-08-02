FROM rust

RUN apt update
RUN apt install -y libmariadb-dev cmake

RUN cargo install sqlx-cli

WORKDIR /usr/src/rusty-retro-messaging
COPY . .

RUN cargo install --path .

EXPOSE 1863 1864 3000
CMD ["rusty-retro-messaging"]