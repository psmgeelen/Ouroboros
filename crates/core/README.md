# ouroboros-core

Account codes, device signatures, sealed epoch keys, partner views, and the on-device cycle model.

The relay may call `verify_detached`. It does not open sealed epoch keys. `decrypt_view` returns a partner view with the current phase and at most two upcoming phases. It does not return cycle history.

The cycle model lives in this crate. The Flutter app mirrors the same update rule.
