# Bootstrap

Bootstrap runs once from a local admin login before GitHub Actions can deploy. Actions cannot create the service account it uses to log in.

## Human IAM

The person who applies this stack needs permission to enable APIs, create service accounts, and grant IAM in:

- `ouroboros-devops`
- `ouroboros-dev-514714`
- `ouroboros-prod-514714`

They also need `roles/owner` or an equivalent set: `serviceusage.serviceUsageAdmin`, `iam.serviceAccountAdmin`, `resourcemanager.projectIamAdmin`, `storage.admin`, and `artifactregistry.admin`.

## Order

1. `gcloud projects describe ouroboros-devops --format='value(projectNumber)'`
2. `terraform -chdir=infra/app/stacks/bootstrap init -input=false`
3. `terraform -chdir=infra/app/stacks/bootstrap apply -var devops_project_number=NUMBER`
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

Attempted on 14 November 2026. `gcloud projects describe ouroboros-devops` could not run: `pieter@geelen.io` needs an interactive `gcloud auth login`, and the other configured accounts are expired, deleted, or not allowed to read that project. The bootstrap stack was not applied. The WIF provider name is not recorded because there is no apply output yet. Re-run `task infra:bootstrap` after `TF_VAR_devops_project_number` is set from `gcloud projects describe ouroboros-devops --format='value(projectNumber)'`.
