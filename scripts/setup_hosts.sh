#!/usr/bin/env bash
set -eo pipefail

public_domain="shor.tt"

setup() {
  if [ -z "$(command -v kubectl)" ]; then
    echo 'Command `kubectl` not found'
    exit 1
  fi

  echo 'If running in minikube, run `minikube tunnel` in separate terminal'

  while true; do
    external_ip="$(kubectl get services | grep client-service | awk '{print $4}')"
    if [ "$enternal_ip" != "<pending>" ]; then
      echo "$external_ip $public_domain" | cat /etc/hosts - | sudo tee /etc/hosts &>/dev/null
      echo "Adding '$public_domain' -> '$external_ip' entry to /etc/hosts"

      exit 0
    fi
    echo "External IP is pending. Retrying in 10 seconds..."
    sleep 10
  done
}

cleanup() {
  sudo sed -i "/$public_domain/d" /etc/hosts
  echo "Removing all '$public_domain' entires from /etc/hosts"
}

case "$1" in
setup)
  setup
  ;;
clean | cleanup)
  cleanup
  ;;
*)
  echo "Usage: $0 (setup|cleanup)"
  exit 1
  ;;
esac
