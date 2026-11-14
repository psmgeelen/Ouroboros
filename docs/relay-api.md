# Relay API

The relay never returns phase text. Wake bodies are `{"wake":"sync"}`.

| Route | Auth | Success | Failure |
| --- | --- | --- | --- |
| `GET /health` | none | 200 | |
| `POST /v1/accounts/enroll` | signature over `enroll:{account_id}:{code_hash}:{public_key}` | 201 | 401 bad signature, 409 duplicate account, 429 rate limit, 503 snapshot upload failed |
| `POST /v1/accounts/restore` | signature over `restore:{account_id}:{code_hash}:{public_key}` and a matching stored hash | 200 | 401 unknown account or hash, 429 rate limit |
| `POST /v1/partnerships` | tracker signature over `partnership:{id}:{tracker}:{invite}` | 201 | 401, 409 duplicate, 429 |
| `POST /v1/partnerships/{id}/confirm` | tracker signature over `confirm:{id}` | 200 | 401, 404 |
| `POST /v1/partnerships/{id}/view` | tracker signature over `upload:{id}:{ciphertext_hex}` and `confirmed: true` | 200 | 409 when `confirmed` is false or the partnership is not confirmed, 401 bad signature |
| `GET /v1/partnerships/{id}/view` | possession of the unguessable partnership id | 200 with ciphertext and sealed key | 404 after revoke |
| `DELETE /v1/partnerships/{id}` | tracker signature over `revoke:{id}` | 204 | 401, 404 |
| `POST /v1/wakes/{id}` | possession of the partnership id | 200 `{"wake":"sync"}` | 404 after revoke |

Revoke deletes the partnership row, the sealed key, and the wake time, then publishes a snapshot. A later fetch is 404.

A stale snapshot generation returns 409 and reloads the previous snapshot instead of keeping the new bytes. A failed snapshot upload returns 503 even though SQLite already committed.
