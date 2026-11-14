# Notifications

The tracker schedules local reminders from the on-device model. Turning partner sharing off clears the partner-facing schedule and leaves those local reminders in place.

Partner wakes are empty. The body is `{"wake":"sync"}` and has no phase label. The phone decrypts phase text only when the sealed key is still present. After revoke, the notification is suppressed.

A notification already visible in the system tray cannot be pulled back.
