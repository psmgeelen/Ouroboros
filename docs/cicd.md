# CI/CD

`main` and pull requests run `.github/workflows/ci.yml`. That workflow does not deploy.

Whitespace, end-of-file, YAML, merge conflicts, large files, Gitleaks, and Grype (`dir:.`, fail on high or critical) are the existing pre-commit hooks. CI installs Grype with `scripts/install-grype.sh`, runs those hooks with `task ci:hooks`, and does not install Rust, Flutter, or Terraform for them. Rustfmt, `terraform fmt`, and `dart format` stay in `.pre-commit-config.yaml` for local commits. The matching CI job checks the same format for the directory it owns. `.grype.yaml` sets the severity cutoff and path excludes.

| Job | When it runs | Directory | Task |
| --- | --- | --- | --- |
| `hooks` | every pull request and every push to `main` | whole tree | `task ci:hooks` |
| `commitlint` | every pull request and every push to `main` | commit messages | `task ci:commitlint` |
| `core` | `crates/core/**`, `Cargo.toml`, `Cargo.lock`, or `rust-toolchain.toml` | `crates/core` | `task ci:core` |
| `relay` | `services/relay/**` or a workspace file the relay crate compiles with | `services/relay` | `task ci:relay` |
| `mobile` | `apps/mobile/**` or `scripts/check-partner-export.sh` | `apps/mobile` | `task ci:mobile` |
| `bootstrap` | `infra/app/stacks/bootstrap/**` or `infra/config/**` | `infra/app/stacks/bootstrap` | `task ci:bootstrap` |
| `relay-stack` | `infra/app/stacks/relay/**` or `infra/config/**` | `infra/app/stacks/relay` | `task ci:relay-stack` |
| `workflows` | `.github/workflows/**` or `scripts/check-ci-workflow.sh` | `.github/workflows` | `task ci:workflows` |

The directory jobs run in parallel. A docs-only change runs `hooks` and `commitlint`.

`task ci` runs the same directory checks locally, also in parallel. It does not repeat the pre-commit whitespace hooks.

`dev` runs `.github/workflows/release-dev.yml`, authenticates as `github-deploy-dev@ouroboros-dev-514714.iam.gserviceaccount.com`, and deploys project `ouroboros-dev-514714`.

`prod` runs `.github/workflows/release-prod.yml`, authenticates as `github-deploy-prod@ouroboros-prod-514714.iam.gserviceaccount.com`, and deploys project `ouroboros-prod-514714`.

Both releases push the relay image only to `europe-west1-docker.pkg.dev/ouroboros-devops/ouroboros/relay`. Each image gets two tags and two labels: the semver (`0.0.1`, or `0.0.1-dev.1` on dev) and the environment (`prod` or `dev`). Cloud Run is updated to the semver tag. There is no Artifact Registry in the environment projects. Terraform state for those applies is `gs://tf-state-ouroboros` with prefix `dev` or `prod`.

## Variables

GitHub environments `dev` and `prod` hold:

- `WIF_PROVIDER`, the Workload Identity provider resource name from bootstrap
- `WIF_SERVICE_ACCOUNT`, the deploy service account email for that environment

A failed WIF login usually means the provider name is wrong, the branch ref is not the one bound to that service account, or the pool attribute condition rejected the repository. The Actions log shows `google-github-actions/auth` before any Terraform runs.

## Signing secrets

These secrets are not in git:

- `ANDROID_KEYSTORE_BASE64`
- `APPLE_CERTIFICATE_BASE64`
- `APPLE_PROVISIONING_PROFILE_BASE64`

`ANDROID_KEYSTORE_BASE64` is absent. The Android release build uses the debug signing config in `apps/mobile/android/app/build.gradle.kts`, and both release workflows upload `ouroboros-android.apk` to the GitHub release for that tag. The production download uses the latest stable release. Apple signing secrets are absent, so the workflows do not upload an IPA.
