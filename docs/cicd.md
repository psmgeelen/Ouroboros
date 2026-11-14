# CI/CD

`main` and pull requests run `.github/workflows/ci.yml`. That workflow does not deploy.

Whitespace, end-of-file, YAML, merge conflicts, large files, Gitleaks, and Grype (`dir:.`, fail on high or critical) are the existing pre-commit hooks. CI installs Grype with `scripts/install-grype.sh` and runs `task ci:hooks`. That GitHub Actions job has no Rust, Flutter, or Terraform, so it skips rustfmt, `terraform fmt`, and `dart format`; the matching directory job still runs format `--check`. Locally, `task ci:hooks` runs those formatters. `.grype.yaml` sets the severity cutoff and path excludes.

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

`task ci` runs the same directory checks locally, also in parallel, including `ci:hooks` (whitespace, YAML, Gitleaks, Grype, and the formatters).

`main` builds artifacts. `dev` and `prod` only promote an artifact that `main` already published. A promotion does not run `docker build` or `flutter build`.

| Workflow | Branch | When it runs | What it does |
| --- | --- | --- | --- |
| `build-relay.yml` | `main` | `services/relay/**`, `crates/core/**`, `Cargo.toml`, `Cargo.lock`, or `rust-toolchain.toml` | Test the relay and push `relay:<commit sha>` |
| `build-android.yml` | `main` | shared Dart, `apps/mobile/android/**`, or the Android pubspec files | Test the app and upload `ouroboros-android-<commit sha>` |
| `build-ios.yml` | `main` | shared Dart, `apps/mobile/ios/**`, or the pubspec files | Upload an IPA, or a skip marker when Apple secrets are absent |
| `promote-relay.yml` | `dev` or `prod` | the relay paths above, or `infra/app/stacks/relay/**` | Move `:dev` or `:prod` onto that SHA and update Cloud Run |
| `promote-android.yml` | `dev` or `prod` | the Android paths above | Attach the main APK to the GitHub Release |
| `promote-ios.yml` | `dev` or `prod` | the iOS paths above | Attach the main IPA to the GitHub Release |

Shared Dart is `apps/mobile/lib/**` and `apps/mobile/test/**`. An Android-only change does not build or promote iOS. An iOS-only change does not build or promote the APK. A docs-only push builds nothing and promotes nothing.

`build-relay.yml` authenticates as `github-build@ouroboros-devops.iam.gserviceaccount.com`. That account can write `europe-west1-docker.pkg.dev/ouroboros-devops/ouroboros/relay` and cannot deploy Cloud Run. The image tag is the commit SHA. The build does not push `:dev` or `:prod`.

`promote-relay.yml` on `dev` authenticates as `github-deploy-dev@ouroboros-dev-514714.iam.gserviceaccount.com` and updates project `ouroboros-dev-514714`. On `prod` it authenticates as `github-deploy-prod@ouroboros-prod-514714.iam.gserviceaccount.com` and updates project `ouroboros-prod-514714`. It pulls `relay:<commit sha>`. If that tag is missing, the job fails and says the image has to be built on `main` first. It retags that digest as `:dev` or `:prod`. When semantic-release tags this commit, it also tags the same digest with that version (`0.0.1`, or `0.0.1-dev.1` on dev) and Cloud Run is updated to the version tag. Otherwise Cloud Run is updated to the environment tag. An infra-only change applies the stack with the environment tag already in the registry and does not retag a digest. There is no Artifact Registry in the environment projects. Terraform state for those applies is `gs://tf-state-ouroboros` with prefix `dev` or `prod`.

The commit on `dev` or `prod` has to be the same commit `main` built. Push `main` and wait for the build before promoting that SHA.

## Variables

GitHub environments `build`, `dev`, and `prod` hold:

- `WIF_PROVIDER`, the Workload Identity provider resource name from bootstrap
- `WIF_SERVICE_ACCOUNT`, `github-build@ouroboros-devops.iam.gserviceaccount.com` on `build`, and the deploy account on `dev` and `prod`

`build` only allows the `main` branch. `dev` only allows the `dev` branch. `prod` only allows the `prod` branch.

A failed WIF login usually means the provider name is wrong, the environment subject is not the one bound to that service account, or the pool attribute condition rejected the repository. The Actions log shows `google-github-actions/auth` before any image push or Terraform run.

## Signing secrets

These secrets are not in git:

- `ANDROID_KEYSTORE_BASE64`
- `APPLE_CERTIFICATE_BASE64`
- `APPLE_PROVISIONING_PROFILE_BASE64`

`ANDROID_KEYSTORE_BASE64` is absent. The Android build on `main` uses the debug signing config in `apps/mobile/android/app/build.gradle.kts`. `promote-android.yml` attaches `ouroboros-android.apk` to the GitHub Release when this commit has a release tag. The production download uses the latest stable release, which the prod promotion publishes. Apple signing secrets are absent, so `build-ios.yml` does not upload an IPA and `promote-ios.yml` does not publish one. Those secrets are read by the `build` environment.
