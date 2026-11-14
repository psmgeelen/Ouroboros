# Threat model

## Tracker

The tracker is the owner of the cycle log. Their phone holds the history, the account code, and the device keys. A person with the unlocked phone can see the log. The app lock is biometric.

## Partner

The partner can see the current shared view while the partnership is active. They do not receive the log, the notes, or the tracker account code. The partner app keeps plaintext and the epoch key in memory only.

## Relay operator

The relay operator can see account hashes, public keys, ciphertext, sealed keys, and wake-up times. They cannot open the sealed epoch key, and they cannot read the cycle history. Wake-up times are metadata. Phase labels are not stored on the relay.

## Stolen phone

A stolen locked phone is protected by the platform keystore and the biometric lock. A stolen unlocked partner phone can show whatever is currently decrypted in memory. After the app locks, that key is dropped.

## Revoke

Revoke deletes the sealed key and the blob from the relay and deletes the partner cache. Revoke stops future reads. Revoke does not erase a screenshot or a copied plaintext.
