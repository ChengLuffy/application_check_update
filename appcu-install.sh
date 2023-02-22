#!/bin/bash

# Replace with your GitHub username and repository name
USER=chengluffy
REPO=application_check_update

# Get arch
if [[ $(uname -m) == "arm64" ]]; then
  arch="aarch64"
else
  arch="x86_64"
fi

# Get the download URL of the latest release asset
download_url=$(curl -s "https://api.github.com/repos/$USER/$REPO/releases/latest" \
| grep "browser_download_url.*tar.gz" \
| grep "$arch" \
| cut -d : -f 2,3 \
| tr -d \" \
| tr -d " ")

# Download the release asset
curl -L -o latest.tar.gz $download_url

# Extract the release asset
tar -zxvf latest.tar.gz

# Mv to /usr/local/bin/
sudo mv appcu /usr/local/bin/

# Cleanup by deleting the downloaded archive
rm latest.tar.gz
