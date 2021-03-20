# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.158.0/containers/rust/.devcontainer/base.Dockerfile

ARG VARIANT
ARG CONTRACT_VERSION

FROM mcr.microsoft.com/vscode/devcontainers/rust:${VARIANT}

ARG CONTRACT_VERSION

ENV CONTRACT_VERSION=${CONTRACT_VERSION}

RUN echo "deb http://ftp.de.debian.org/debian/ testing main contrib non-free" >> /etc/apt/sources.list

RUN apt-get update \
  && export DEBIAN_FRONTEND=noninteractive \
  && apt-get install -y cmake pkg-config libssl-dev git build-essential clang libclang-dev gcc curl vim \
  && apt-get install -y binaryen/testing