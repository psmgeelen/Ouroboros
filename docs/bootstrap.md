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

`infra/config/hooks/terraform.rb` runs before Terraspace `init`, `plan`, and `apply`. It sets `GOOGLE_OAUTH_ACCESS_TOKEN` from `gcloud auth print-access-token` and, for the bootstrap stack, sets `TF_VAR_devops_project_number` from `gcloud projects describe ouroboros-devops`. GitHub Actions skips this hook and uses Workload Identity. `task infra:bootstrap` sources `scripts/gcp-user-auth.sh` for the same token refresh, because that task calls Terraform directly.

3. State is in `gs://tf-state-ouroboros` in `ouroboros-devops`. Prefixes are `bootstrap`, `dev`, and `prod`. That bucket is created outside this stack. Bootstrap does not create another state bucket.
4. Copy the `wif_provider` output into the GitHub environments `build`, `dev`, and `prod` as `WIF_PROVIDER`.
5. Set `WIF_SERVICE_ACCOUNT` to the account for that environment. `build` uses the registry writer. `dev` and `prod` use the deploy accounts.
6. Store Android and Apple signing material as GitHub secrets. Do not commit them.

## Accounts that must exist before Actions runs

- `github-build@ouroboros-devops.iam.gserviceaccount.com` trusts GitHub environment `build` (`repo:psmgeelen@30153353/Ouroboros@1457826099:environment:build`). That environment only allows the `main` branch. The account can write the one Artifact Registry repository and has no Cloud Run roles.
- `github-deploy-dev@ouroboros-dev-514714.iam.gserviceaccount.com` trusts GitHub environment `dev` (`repo:psmgeelen@30153353/Ouroboros@1457826099:environment:dev`). That environment only allows the `dev` branch.
- `github-deploy-prod@ouroboros-prod-514714.iam.gserviceaccount.com` trusts GitHub environment `prod` the same way, and that environment only allows the `prod` branch.
- `relay-runtime-dev@ouroboros-dev-514714.iam.gserviceaccount.com`
- `relay-runtime-prod@ouroboros-prod-514714.iam.gserviceaccount.com`

Bootstrap does not create Cloud Run, the load balancer, or snapshot buckets.

There is one Artifact Registry, `europe-west1-docker.pkg.dev/ouroboros-devops/ouroboros`. Dev and prod do not have a registry. `main` pushes the immutable commit SHA. Promotion adds the semver tag and the environment tag, for example `1.2.3` and `prod`.

## Apply record

Applied on 14 November 2026 with local admin credentials for `pieter@geelen.io`. Terraform used `GOOGLE_OAUTH_ACCESS_TOKEN` from `gcloud auth print-access-token` because the application-default credential still returned `invalid_rapt`.

State for this stack is `gs://tf-state-ouroboros` prefix `bootstrap`. Dev relay state was copied to prefix `dev` before `gs://ouroboros-devops-tfstate` was removed.

`devops_project_number` is `461942996777`.

The Workload Identity provider name is `projects/461942996777/locations/global/workloadIdentityPools/github/providers/github`. Set that as `WIF_PROVIDER` in the GitHub environments `build`, `dev`, and `prod`. Set `WIF_SERVICE_ACCOUNT` to `github-build@ouroboros-devops.iam.gserviceaccount.com` on `build`, and to the deploy account on `dev` and `prod`.
