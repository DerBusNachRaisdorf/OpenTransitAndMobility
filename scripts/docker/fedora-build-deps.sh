#!/bin/bash

# strict bash mode
set -euo pipefail

dnf --nodocs -y upgrade-minimal
dnf --nodocs -y install --setopt=install_weak_deps=False protobuf-compiler

# clear cache to keep docker image size small
dnf clean all
