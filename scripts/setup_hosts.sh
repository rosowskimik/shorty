#!/usr/bin/env bash
set -euo pipefail

public_domain="shor.tt"

if [ -z "$(command -v kubectl)" ]; then
    echo "Command \`kubectl\` not found"
    exit 1
fi

_cleanup_hostfile() {
    echo Removing all \'"$public_domain"\' entires from /etc/hosts
    sed  "/$public_domain/d" /etc/hosts
}

trap _cleanup_hostfile EXIT

echo "Adding \'$public_domain\' entry to /etc/hosts"
echo "If running in minikube, run \`minikube tunnel\` in separate terminal"

while true; do
    external_ip="$(kubectl get services | grep client-service | awk '{print $4}')"
    if [ "$enternal_ip" != "<pending>" ]; then
        echo "$external_ip $public_domain" >> /etc/hosts
	echo "Entry added"
	exit 0
    fi
    echo "External IP is pending. Retrying in 10 seconds..."
    sleep 10
done
