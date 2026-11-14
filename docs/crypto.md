# Crypto

Account codes are 16 digits from a CSPRNG. The relay stores `hash_account_code`, an HMAC-SHA256 with a pepper. The raw code is not in the hash.

Device identities have an Ed25519 signing key and an X25519 agreement key. Uploads are signed with the signing key. Epoch keys are sealed to the agreement key with an ephemeral X25519 exchange and XChaCha20-Poly1305.

A partner view is encrypted under a fresh epoch key. The seal is stored beside the ciphertext. The next publish uses a new epoch key. An old key does not decrypt a new blob, and a new key does not decrypt the old blob.

## Tests and the threats they cover

- `wrong_account_code_does_not_verify` covers a guessed account code. The hash does not contain the code, and a wrong code is rejected.
- `deleted_seal_does_not_decrypt` covers revoke and a stripped seal. Ciphertext without the seal does not open.
- `bad_signature_is_rejected` covers a blob signed by a different device. Provenance fails closed.
- `epoch_rotation_isolates_blobs` covers a stolen previous epoch key. It cannot read a later view.
- `history_payload_is_rejected` covers a payload that smuggles cycle history, notes, logs, or an account code into a partner view.
- `short_code_is_stable` covers the pairing check. The same transcript always shows the same six digits, and a changed byte changes them.
