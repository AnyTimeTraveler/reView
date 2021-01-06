#!/usr/bin/env zsh

name="$(pwd | rev |cut -d/ -f1 | rev)"
host="remarkable"
arch="armv7-unknown-linux-musleabihf"
#arch="armv7-unknown-linux-gnueabihf"

echo "Checking if Docker is running..."
if ! systemctl status docker.service > /dev/null; then
  echo "Done"
  echo "Starting Docker..."
  sudo systemctl start docker.service
fi
echo "Done"

echo "Compiling..."
cross build --target "$arch" --release || exit 1
echo "Done"

echo "Killing last process..."
# shellcheck disable=SC2029
ssh "$host" "killall $name"
echo "Done"

echo "Checking if /opt/ directory exists in device..."
ssh "$host" "mkdir -p /opt/" || exit 1
echo "Done"

echo "Copying to device..."
scp "./target/$arch/release/$name" "$host:/opt/$name" || exit 1
echo "Done"

echo "Running..."
# shellcheck disable=SC2029
ssh "$host" "RUST_BACKTRACE=full /opt/$name" || exit 1
echo "Done"
