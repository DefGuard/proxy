#!/usr/bin/env bash
set -e

echo "Updating apt repositories..."
sudo apt update

echo "Installing Defguard Proxy package..."
sudo dpkg -i /tmp/defguard-proxy.deb 

echo "Cleaning up..."
sudo rm -f /tmp/defguard-proxy.deb

echo "Defguard Proxy installation completed successfully."
