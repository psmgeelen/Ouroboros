#!/usr/bin/env bash
# Refresh the gcloud user token before Terraform.
# Application Default Credentials can keep a stale reauth proof (invalid_rapt)
# after `gcloud auth login`. GitHub Actions uses Workload Identity instead.
set -euo pipefail

if [ "${GITHUB_ACTIONS:-}" = "true" ]; then
  return 0 2>/dev/null || exit 0
fi

token="$(gcloud auth print-access-token)"
if [ -z "$token" ]; then
  echo "gcloud auth print-access-token returned an empty token" >&2
  exit 1
fi

number="${TF_VAR_devops_project_number:-}"
if [ "${GCP_AUTH_WITH_PROJECT_NUMBER:-}" = "1" ] && [ -z "$number" ]; then
  number="$(gcloud projects describe ouroboros-devops --format='value(projectNumber)')"
  if [ -z "$number" ]; then
    echo "could not read the ouroboros-devops project number" >&2
    exit 1
  fi
fi

if [ "${1:-}" = "print" ]; then
  printf '%s\n' "$token"
  printf '%s\n' "$number"
  exit 0
fi

export GOOGLE_OAUTH_ACCESS_TOKEN="$token"
if [ -n "$number" ]; then
  export TF_VAR_devops_project_number="$number"
fi
