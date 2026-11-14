# ouroboros-relay

Blind HTTP relay. It stores peppered account hashes, public keys, ciphertext, sealed keys, and wake-up times. It has no function that opens a sealed key.

## Local run

```bash
SQLITE_PATH=/tmp/relay.sqlite SNAPSHOT_PATH=/tmp/relay.snapshot cargo run -p ouroboros-relay
```

`GET /health` returns 200.

## Environment

- `SQLITE_PATH` is the local SQLite file. In the container this is `/tmp/relay.sqlite`. `/tmp` is the only writable path.
- `SNAPSHOT_PATH` is the snapshot file. In production the same bytes are uploaded to the private Cloud Storage bucket created by the relay stack. The object generation must match before the upload replaces it. The HTTP handler returns success only after that upload.

## Image

```bash
docker build -f services/relay/Dockerfile -t ouroboros-relay .
```

The image is distroless and runs as `nonroot`. On `main`, `build-relay.yml` pushes
`europe-west1-docker.pkg.dev/ouroboros-devops/ouroboros/relay:<commit sha>`.
`promote-relay.yml` on `dev` or `prod` retags that digest as `:dev` / `:prod` and,
when semantic-release tags the commit, as `:<version>`.
