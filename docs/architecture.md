# Architecture

The tracker phone is the only place that stores cycle history. The partner phone receives a narrow partner view: the current phase, at most two upcoming phases, and a confidence value. A blind relay on Cloud Run carries that view when the phones are apart.

```
tracker phone -> Cloud Armor -> application load balancer -> Cloud Run relay -> Cloud Storage snapshot
partner phone -> Cloud Armor -> application load balancer -> Cloud Run relay
```

## Three stores

The tracker phone keeps the log in an encrypted database. The database key, the account code, and the device identity key stay in the platform keystore.

The partner phone may keep the latest ciphertext blob in a separate file. The epoch key and the decrypted view stay in memory.

The relay keeps one SQLite database. After each commit it uploads a snapshot of that file to a private Cloud Storage bucket, and only then returns success. Cloud SQL is not used: the data is small and the smallest Postgres instance would be an always-on machine. DuckDB is not used because this workload is small transactional updates. Litestream is not used because it would copy the log after the response, so a revoke could be acknowledged and then lost.

## Identity and keys

Every install has a 16-digit anonymous account code. The relay stores a peppered hash of that code and a public key. Partner views are encrypted with an epoch key. That epoch key is sealed to the partner public key (a sealed epoch key). The relay stores the ciphertext and the seal. It has no decryption code. Each publish uses a new epoch key.

## Deploy projects

Runtime deploys use these project IDs:

- `ouroboros-devops` holds Artifact Registry, the Workload Identity pool, and the Terraform state bucket.
- `ouroboros-dev-514714` is the dev relay.
- `ouroboros-prod-514714` is the prod relay.

`geelen.io` and `ouroboros` are not deploy targets.
