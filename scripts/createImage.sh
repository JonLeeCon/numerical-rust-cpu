#!/busybox/sh

DOCKERFILE="$1"
FINAL_IMAGE="$2"
BASE_IMAGE_ARG=""
if [[ $# -gt 2 ]]; then
    BASE_IMAGE_ARG="--build-arg BASE_IMAGE=\"$3\""
fi

mkdir -p /kaniko/.docker
echo "{\"auths\":{\"${CI_REGISTRY}\":{\"auth\":\"$(printf "%s:%s" "${CI_REGISTRY_USER}" "${CI_REGISTRY_PASSWORD}" | base64 | tr -d '\n')\"}}}" > /kaniko/.docker/config.json
/kaniko/executor \
    --cache \
    --cache-copy-layers \
    --cache-ttl 48h \
    --cleanup \
    --push-retry 1 \
    --use-new-run \
    ${BASE_IMAGE_ARG} \
    --context "${CI_PROJECT_DIR}" \
    --dockerfile "${CI_PROJECT_DIR}/docker/Dockerfile.${DOCKERFILE}" \
    --destination "${FINAL_IMAGE}:${TAG_PIPELINE_SELF}" \
    --destination "${FINAL_IMAGE}:${TAG_PIPELINE_LATEST}"


