# Android release signing

Every Android update must be signed with the same key. A phone refuses an update signed with a
different key, and the only way round that is to uninstall, which loses the user's learned words
and settings. So the key is permanent: keep a backup outside GitHub (a password manager, or an
offline copy), and never commit it to this repository.

The Release workflow signs the app with four repository secrets
(Settings → Secrets and variables → Actions):

| Secret | What it holds |
|---|---|
| `ANDROID_KEYSTORE_B64` | The keystore file (`ekushe-next-release.jks`), base64-encoded: `base64 -w0 ekushe-next-release.jks` |
| `ANDROID_KEYSTORE_PASSWORD` | The keystore's password |
| `ANDROID_KEY_ALIAS` | `ekushe-next` |
| `ANDROID_KEY_PASSWORD` | The key's password (the same as the keystore's) |

If `ANDROID_KEYSTORE_B64` is missing, the Android job fails and no release is published, so a
release never goes out unsigned.

To build a signed APK on your own machine instead, put the same values in `android/keystore.properties`
(ignored by git):

```
storeFile=/path/to/ekushe-next-release.jks
storePassword=…
keyAlias=ekushe-next
keyPassword=…
```

The Play Store uses this key as the upload key; with Play App Signing, Google keeps the key that
phones see, and a lost upload key can be reset through Play Console support.
