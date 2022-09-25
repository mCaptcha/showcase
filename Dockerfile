##FROM rust:latest as planner
##RUN cargo install cargo-chef
##WORKDIR /src
##COPY . /src/
##RUN cargo chef prepare --recipe-path recipe.json
##
##FROM rust:latest as cacher
##WORKDIR /src/
##RUN cargo install cargo-chef
##COPY --from=planner /src/recipe.json recipe.json
##RUN cargo chef cook --release --recipe-path recipe.json
#
#FROM rust:latest as rust
#WORKDIR /src
#COPY . .
##COPY --from=cacher /src/target target
#RUN cargo --version
#RUN cargo build --release
#
#FROM debian:bullseye as mcaptcha-showcase
#LABEL org.opencontainers.image.source https://github.com/mCaptcha/dos
#RUN useradd -ms /bin/bash -u 1001 mcaptcha-showcase
#WORKDIR /home/mcaptcha-showcase
#COPY --from=rust /src/target/release/mcaptcha-showcase /usr/local/bin/
#COPY --from=rust /src/config/default.toml /etc/mcaptcha-showcase/config.toml
#USER mcaptcha-showcase
#CMD [ "/usr/local/bin/mcaptcha-showcase" ]


FROM rust:slim as rust
WORKDIR /src
RUN apt-get update && apt-get install -y git pkg-config libssl-dev make
RUN mkdir src && echo "fn main() {}" > src/main.rs
COPY Cargo.toml .
RUN sed -i '/.*build.rs.*/d' Cargo.toml
COPY Cargo.lock .
RUN cargo build --release || true
COPY . /src
RUN cargo build --release

FROM debian:bullseye as mcaptcha-showcase
LABEL org.opencontainers.image.source https://github.com/mCaptcha/dos
RUN useradd -ms /bin/bash -u 1001 mcaptcha-showcase
WORKDIR /home/mcaptcha-showcase
COPY --from=rust /src/target/release/mcaptcha-showcase /usr/local/bin/
COPY --from=rust /src/config/default.toml /etc/mcaptcha-showcase/config.toml
USER mcaptcha-showcase
CMD [ "/usr/local/bin/mcaptcha-showcase" ]
