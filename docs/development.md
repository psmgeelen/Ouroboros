# Development

## Branches

- `main` is local development and pull-request checks. CI runs tests and lint. It does not build an image, push a tag, or deploy.
- `dev` rolls out project `ouroboros-dev-514714` and publishes prerelease tags.
- `prod` rolls out project `ouroboros-prod-514714` and publishes release tags.

## Commits

Commits follow Conventional Commits: `feat:`, `fix:`, `docs:`, `style:`, `refactor:`, `perf:`, `test:`, `build:`, `ci:`, `chore:`, and `revert:`.

## GCP project IDs

The deploy project IDs are recorded in [architecture.md](architecture.md):

- `ouroboros-devops`
- `ouroboros-dev-514714`
- `ouroboros-prod-514714`

`geelen.io` (project number `999278164964`) and `ouroboros` (project number `153931698106`) stay outside the pipeline.
