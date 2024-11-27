#!/bin/bash

printf "### Use the following "
grep password: "${DOCKER_HOME}/.config/code-server/config.yaml"
echo "### If using Docker, mounted files must be chown'd to $(id -u):$(id -g)"
code-server "${DOCKER_HOME}/exercises"


