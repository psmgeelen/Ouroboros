# Bootstrap

Bootstrap runs once from a local admin login before GitHub Actions can deploy. Actions cannot create the service account it uses to log in.

## Human IAM

The person who applies this stack needs permission to enable APIs, create service accounts, and grant IAM in:

- `ouroboros-devops`
- `ouroboros-dev-514714`
- `ouroboros-prod-514714`

They also need `roles/owner` or an equivalent set: `serviceusage.serviceUsageAdmin`, `iam.serviceAccountAdmin`, `resourcemanager.projectIamAdmin`, `storage.admin`, and `artifactregistry.admin`.

## Order

1. `gcloud auth login` as an admin on the three projects.
2. `task infra:bootstrap`, or `terraspace up bootstrap` from `infra/`.

`infra/config/hooks/terraform.rb` runs before Terraspace `init`, `plan`, and `apply`. It sets `GOOGLE_OAUTH_ACCESS_TOKEN` from `gcloud auth print-access-token` and, for the bootstrap stack, sets `TF_VAR_devops_project_number` from `gcloud projects describe ouroboros-devops`. GitHub Actions skips this hook and uses Workload Identity. `task infra:bootstrap` sources `scripts/gcp-user-auth.sh` for the same token refresh, because that task calls Terraform directly and keeps the existing local state file.
4. Copy the `wif_provider` output into the GitHub environments `dev` and `prod` as `WIF_PROVIDER`.
5. Set `WIF_SERVICE_ACCOUNT` to the deploy account for that environment.
6. Store Android and Apple signing material as GitHub secrets. Do not commit them.
7. Move discussion of state to `gs://ouroboros-devops-tfstate` with prefixes `dev/` and `prod/`. The first apply can use the local backend. Later relay applies pass `-backend-config=prefix=dev` or `prefix=prod`.

## Accounts that must exist before Actions runs

- `github-deploy-dev@ouroboros-dev-514714.iam.gserviceaccount.com` trusts only `refs/heads/dev`
- `github-deploy-prod@ouroboros-prod-514714.iam.gserviceaccount.com` trusts only `refs/heads/prod`
- `relay-runtime-dev@ouroboros-dev-514714.iam.gserviceaccount.com`
- `relay-runtime-prod@ouroboros-prod-514714.iam.gserviceaccount.com`

Bootstrap does not create Cloud Run, the load balancer, or snapshot buckets.

## Apply record

Applied on 14 November 2026 with local admin credentials for `pieter@geelen.io`. Terraform used `GOOGLE_OAUTH_ACCESS_TOKEN` from `gcloud auth print-access-token` because the application-default credential still returned `invalid_rapt`.

`devops_project_number` is `461942996777`.

The Workload Identity provider name is `projects/461942996777/locations/global/workloadIdentityPools/github/providers/github`. Set that as `WIF_PROVIDER` in the GitHub environments `dev` and `prod`. Set `WIF_SERVICE_ACCOUNT` to the deploy account for that environment.
