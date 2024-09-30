#!/bin/bash

# strict bash mode
set -euo pipefail

# apt-get without manual input
export DEBIAN_FRONTEND=noninteractive

apt-get update
apt-get -y upgrade

# required for all debian images:
apt-get -y install --no-install-recommends protobuf-compiler

# only required for *-slim images,
# as those might not come with the following packages preinstalled:
apt-get -y install --no-install-recommends pkg-config
apt-get -y install --no-install-recommends libssl-dev

# clear apt cache and delete index files to keep docker image size small
apt-get clean
rm -rf /var/lib/apt/lists/*
