# A Fedora 25 BDCS API Container
FROM fedora:25
LABEL maintainer Will Woods <wwoods@redhat.com>

# Define our workdir
WORKDIR /bdcs-api-rs

# Volumes for database and recipe storage.
VOLUME /mddb /recipes /mockfiles

# Install build requirements
RUN dnf --setopt=deltarpm=0 --assumeyes install \
    gcc make cmake openssl-devel libcurl-devel sqlite sqlite-devel

# Install the Rust toolchain
ENV RUST_TOOLCHAIN="nightly-2017-04-17"
RUN curl https://sh.rustup.rs -sSf \
  | sh -s -- -y --default-toolchain $RUST_TOOLCHAIN
ENV PATH="/root/.cargo/bin:${PATH}"

# Enable incremental building
ENV CARGO_INCREMENTAL=1

# Fetch & build dependencies
COPY Cargo.toml Cargo.lock ./
# Build a dummy lib as a hacky way to do `cargo build --dependencies-only`
# (see https://github.com/rust-lang/cargo/issues/2644)
RUN mkdir src && touch src/lib.rs && \
    cargo build --release --lib && \
    cargo clean --release --package bdcs

# Copy the source in and build for real
COPY src src/
RUN cargo install --path . --root /usr/local

# Copy the examples into place
COPY examples/recipes /recipes/

# We're ready to go!
ENTRYPOINT ["/usr/local/bin/bdcs-api-server", \
            "--host", "0.0.0.0", \
            "--port", "4000", \
            "/mddb/metadata.db", \
            "/recipes"]
EXPOSE 4000
