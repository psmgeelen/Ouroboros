# Pairing

1. Both phones already have an account code and a device key. The account code is not in the QR payload.
2. The tracker creates a partnership and shows a QR code with the relay address, partnership id, tracker public key, and a one-time invite.
3. The partner scans. Both phones show a six-digit short code from the handshake transcript.
4. The tracker confirms the codes match. Only then does the app upload a signed, sealed partner view.
5. The partner still cannot see the cycle log, notes, or the tracker account code.
