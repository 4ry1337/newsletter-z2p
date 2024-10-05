#!/usr/bin/env bash
set -x
set -eo pipefail

EMAIL="yskak.rakhat@gmail.com"
PWD="password"
PGADMIN_PORT=5050

CONTAINER_NAME="pgadmin"
docker run \
  --name "${CONTAINER_NAME}" \
  --env PGADMIN_DEFAULT_EMAIL=${EMAIL} \
  --env PGADMIN_DEFAULT_PASSWORD=${PWD} \
  --publish ${PGADMIN_PORT}:80 \
  --detach \
  dpage/pgadmin4

>&2 echo "PG admin is running on port ${PGADMIN_PORT}"
