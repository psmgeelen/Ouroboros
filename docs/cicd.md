# CI/CD

`main` runs `.github/workflows/ci.yml`. That workflow runs `task ci` and does not deploy.

`dev` runs `.github/workflows/release-dev.yml`, authenticates as `github-deploy-dev@ouroboros-dev-514714.iam.gserviceaccount.com`, and deploys project `ouroboros-dev-514714`.

`prod` runs `.github/workflows/release-prod.yml`, authenticates as `github-deploy-prod@ouroboros-prod-514714.iam.gserviceaccount.com`, and deploys project `ouroboros-prod-514714`.

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

Android signing secret is absent. Apple signing secrets are absent. The release workflows skip the mobile publish step instead of shipping an unsigned build. `ouroboros-android.apk` and `ouroboros-ios.ipa` are the filenames those steps upload when the secrets exist.
