#!/bin/sh

# Durchsucht alle Verzeichnisse nach Cargo.toml
find . -type f -name "Cargo.toml" | while read -r cargo_toml; do
  # Ermittelt das Verzeichnis, in dem Cargo.toml liegt
  dir=$(dirname "$cargo_toml")

  # Erstellt das src Verzeichnis, falls es nicht existiert
  src_dir="$dir/src"
  if [ ! -d "$src_dir" ]; then
    mkdir "$src_dir"
    echo "Erstellt: $src_dir"
  fi

  # Erstellt die lib.rs Datei im src Verzeichnis, falls sie nicht existiert
  lib_file="$src_dir/lib.rs"
  if [ ! -f "$lib_file" ]; then
    touch "$lib_file"
    echo "Erstellt: $lib_file"
  fi
done

echo "Fertig!"
