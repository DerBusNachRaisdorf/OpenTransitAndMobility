#!/bin/bash

# strict bash mode
set -euo pipefail

# apt-get without manual input
export DEBIAN_FRONTEND=noninteractive

apt-get update
apt-get -y upgrade

# only required for *-slim images,
# as those might not come with the following packages preinstalled:
apt-get -y install --no-install-recommends libssl3
apt-get -y install --no-install-recommends ca-certificates

# timezone
apt-get -y install tzdata && \
    ln -fs /usr/share/zoneinfo/Europe/Berlin /etc/localtime && \
    dpkg-reconfigure -f noninteractive tzdata


# clear apt cache and delete index files to keep docker image size small
apt-get clean
rm -rf /var/lib/apt/lists/*
