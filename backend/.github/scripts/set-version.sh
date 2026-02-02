#!/bin/bash
# filepath: .github/scripts/set-version.sh
set -e 

REGISTRY=$1
HARBOR_PROJECT=$2
IMAGE_NAME=$3
HARBOR_USERNAME=$4
HARBOR_PASSWORD=$5

# Harbor API를 통해 기존 태그 목록 가져오기
TAGS=$(curl -s -u "$HARBOR_USERNAME:$HARBOR_PASSWORD" \
                    "https://$REGISTRY/api/v2.0/projects/$HARBOR_PROJECT/repositories/$IMAGE_NAME/artifacts" \
                    | jq -r '.[].tags[].name' 2>/dev/null || echo "")
# 최신 태그 결정
if [ -z "$TAGS" ]; then
    NEW_TAG="v0.1.0"
else
    LATEST_TAG=$(echo "$TAGS" | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+$' | sort -V | tail -n 1)
    if [ -z "$LATEST_TAG" ]; then
        NEW_TAG="v0.1.0"
    else
        MAJOR=$(echo "$LATEST_TAG" | cut -d. -f1 | sed 's/v//')
        MINOR=$(echo "$LATEST_TAG" | cut -d. -f2)
        PATCH=$(echo "$LATEST_TAG" | cut -d. -f3)
        PATCH=$((PATCH + 1))
        NEW_TAG="v$MAJOR.$MINOR.$PATCH"
    fi
fi
