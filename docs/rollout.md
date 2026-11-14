# Rollout

`TS_ENV=dev` selects project `ouroboros-dev-514714` through `infra/app/stacks/relay/dev.tfvars`.

`TS_ENV=prod` selects project `ouroboros-prod-514714` through `infra/app/stacks/relay/prod.tfvars`.

Dev cannot apply prod. The dev workflow calls `task infra:up ENV=dev` and does not reference the prod project. The prod workflow calls `task infra:up ENV=prod` and does not reference the dev project.

The image URL and the runtime service account email are inputs. The relay stack does not create the GitHub deploy accounts.

## Dev deploy record

Not rolled out yet. The Cloud Run URL and image tag are recorded here after the first successful dev apply.
