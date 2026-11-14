# Partner mode

A partner sees the current phase, at most two upcoming phases, and the tracker key fingerprint.

The epoch key and the decrypted view are memory-only. They are dropped when the app locks. The disk cache is ciphertext.

Revoke sequence:

1. The tracker signs a delete for the partnership.
2. The relay removes the row, the sealed key, and the wake time, then publishes a new snapshot.
3. The partner fetch returns 404.
4. The partner app deletes its ciphertext cache and shows "Access ended".
5. A second fetch does not refill the cache, because the sealed key is gone.
