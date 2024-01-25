# Use an image with a specific version of Rust.
FROM lukemathwalker/cargo-chef:0.1.62-rust-1.75-slim-buster as planner

WORKDIR /deepdecipher
COPY . .
RUN cargo chef prepare --recipe-path recipe.json


FROM lukemathwalker/cargo-chef:0.1.62-rust-1.75-slim-buster as backend-build
WORKDIR /deepdecipher

# Build dependencies
COPY  --from=planner /deepdecipher/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build project
COPY . .
RUN cargo build --release

# our final base
FROM debian:buster-slim as backend-prod

# copy the build artifact from the build stage
COPY --from=backend-build /deepdecipher/target/release/server /deepdecipher-backend

# set the startup command to run your binary
CMD ["./deepdecipher-backend", "data.db", "--url", "0.0.0.0"]