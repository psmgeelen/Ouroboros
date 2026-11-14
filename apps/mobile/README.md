# Ouroboros mobile

Flutter app for iOS and Android. The theme is Material 3 seeded with forest green `#228B22`.

## Run

```bash
cd apps/mobile
flutter run
```

## Sandboxes

- `tracker` holds the encrypted log. The key is not stored in that file.
- `partner` holds ciphertext only. Plaintext and the epoch key stay in memory and are dropped on lock.

## Logo

`assets/brand/logo.svg` is not present. The licensed iStock file has not been added. Launcher icons are the temporary Flutter defaults. Do not download the iStock preview.
