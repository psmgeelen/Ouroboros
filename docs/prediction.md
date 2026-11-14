# Prediction

The model keeps a mean and a variance for cycle length. Each observed cycle updates both. The next window is the mean plus or minus at least one day, so a single cycle does not produce a zero-width prediction.

Feedback:

- `started` records another observation at the current mean.
- `not yet` moves the mean one day later and increases variance. It does not shrink variance to zero.
- `started earlier` moves the mean one day earlier.

The prediction screen shows this disclaimer:

This is an estimate from your own logs. Ouroboros is not a medical device.
