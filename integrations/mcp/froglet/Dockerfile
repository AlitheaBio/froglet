# check=skip=InvalidDefaultArgInFrom
# Froglet MCP Server Docker Image
#
# Build from repo root:
#   docker build -f integrations/mcp/froglet/Dockerfile \
#     --build-arg FROGLET_NODE_IMAGE=<provider-image@sha256:digest> \
#     -t froglet-mcp .
#
# The marketplace_publish action shells out to the froglet-node binary, so the
# image bundles it by copying from a provider image. FROGLET_NODE_IMAGE must
# point at the matching froglet-provider image; release builds pass the
# immutable digest resolved after pushing it earlier in the pipeline:
#   docker build -f integrations/mcp/froglet/Dockerfile \
#     --build-arg FROGLET_NODE_IMAGE=ghcr.io/armanas/froglet-provider@sha256:<digest> \
#     -t froglet-mcp .
#
# Run:
#   docker run --rm -i \
#     -e FROGLET_PROVIDER_URL=http://host.docker.internal:8080 \
#     -e FROGLET_RUNTIME_URL=http://host.docker.internal:8081 \
#     froglet-mcp

ARG FROGLET_NODE_IMAGE
FROM ${FROGLET_NODE_IMAGE} AS froglet-node-src

FROM node:22-slim AS build
WORKDIR /build

# Copy shared library (dependency of MCP server)
COPY integrations/shared/ ./shared/

# Install production dependencies only
COPY integrations/mcp/froglet/package.json integrations/mcp/froglet/package-lock.json ./mcp/
RUN cd mcp && npm ci --production --ignore-scripts

# Copy MCP server source
COPY integrations/mcp/froglet/server.js ./mcp/
COPY integrations/mcp/froglet/lib/ ./mcp/lib/

FROM node:22-slim
LABEL org.opencontainers.image.title="froglet-mcp"
LABEL org.opencontainers.image.description="MCP server for Froglet services, compute, and marketplace"
LABEL org.opencontainers.image.url="https://github.com/AlitheaBio/froglet"

RUN groupadd --gid 10001 froglet && \
    useradd --uid 10001 --gid 10001 --no-create-home froglet

# froglet-node uses rustls with the platform verifier, which reads the
# system CA store. Its locked Python publication path needs a local CPython
# interpreter to attest and package inline Python services without pip.
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates python3 tini && \
    rm -rf /var/lib/apt/lists/*

# froglet-node is required by marketplace_publish (execFile from
# integrations/shared/froglet-lib/marketplace-publish.js). Same-version copy
# from the provider image; both bases are Debian bookworm, so the glibc ABI
# matches.
COPY --from=froglet-node-src /usr/local/bin/froglet-node /usr/local/bin/froglet-node
ENV FROGLET_NODE_BIN=/usr/local/bin/froglet-node

WORKDIR /
# Runtime imports resolve ../../../shared/... from /app/mcp/lib/*.js, which lands at /shared.
COPY --from=build --chown=froglet:froglet /build/shared/ ./shared/
WORKDIR /app
COPY --from=build --chown=froglet:froglet /build/mcp/ ./mcp/

USER froglet
WORKDIR /app/mcp

ENTRYPOINT ["/usr/bin/tini", "--", "node", "server.js"]
