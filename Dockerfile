### INSTALL PACKAGES
FROM rust:latest AS deps
# set the working directory inside the container
WORKDIR /data_portability_api_mocker
# this takes a while due to crates index update, so we do it first
RUN cargo install cargo-chef --locked

### COMPUTE THE RECIPE FILE
FROM deps AS planner
COPY . .
# build a recipe (locl-file) that captures the set of information required to build the dependencies
RUN cargo chef prepare --recipe-path recipe.json

### CACHE THE DEPENDENCIES AND BUILD THE BINARY
# the builder stage does not contribute to the size of the final image, it is an intermediate state
# and it is discarded at the end of the build
FROM deps AS builder
COPY --from=planner /data_portability_api_mocker/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
# As long as your dependencies do not change the recipe.json file will stay the same,
# therefore the outcome of cargo chef cook --recipe-path recipe.json will be cached
RUN cargo chef cook --recipe-path recipe.json
# Copy the entire project into the /data_portability_api_mocker folder inside the container
# Exclude the files and folders specified in .dockerignore
COPY . .
RUN cargo build --bin data_portability_api_mocker

### RUN THE BINARY IN THE RUNTIME ENVIRONMENT (we don't need the rust toolchain to run the binary)
FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
# we only copy the compiled binary from the builder environment to the runtime environment
COPY --from=builder /data_portability_api_mocker/target/debug/data_portability_api_mocker data_portability_api_mocker
RUN mkdir configuration
RUN chmod 1777 -R configuration
# Run the DP API mocker 
CMD ["./data_portability_api_mocker"]

