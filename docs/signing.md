# Release signing

The tagged-release workflow refuses to build or publish when any signing credential is absent. Configure these encrypted GitHub repository secrets before pushing a version tag.

## Windows Authenticode

- `WINDOWS_SIGNING_CERTIFICATE_BASE64`: Base64-encoded PFX containing a trusted code-signing certificate and private key.
- `WINDOWS_SIGNING_CERTIFICATE_PASSWORD`: PFX password.

The certificate subject determines the publisher name shown by Windows and should identify Young Hyun Chi. The workflow signs both x86-64 and ARM64 executables with SHA-256, requests a trusted timestamp, and runs `signtool verify` before packaging. A trusted certificate prevents an unsigned or unknown-publisher result; Microsoft SmartScreen reputation remains controlled by Microsoft and may take time to accumulate for a new certificate.

Encode a PFX in PowerShell with `[Convert]::ToBase64String([IO.File]::ReadAllBytes("certificate.pfx"))`. Never commit the PFX, its password, or the encoded value.

## macOS Developer ID and notarization

- `MACOS_SIGNING_CERTIFICATE_BASE64`: Base64-encoded PKCS #12 export of a Developer ID Application certificate and private key.
- `MACOS_SIGNING_CERTIFICATE_PASSWORD`: PKCS #12 password.
- `MACOS_SIGNING_IDENTITY`: Exact Keychain identity, such as the Developer ID Application identity issued to Young Hyun Chi.
- `APPLE_ID`: Apple Developer account identifier used by `notarytool`.
- `APPLE_TEAM_ID`: Apple Developer Team ID.
- `APPLE_APP_SPECIFIC_PASSWORD`: App-specific password for notarization.

The workflow creates a `com.cyhdev.eu5-location-filter` application bundle, applies hardened-runtime signing, notarizes the archive with Apple, staples the ticket, and verifies it with both `codesign` and Gatekeeper before upload. Encode the PKCS #12 file with `base64 -i certificate.p12 | pbcopy` on macOS.

## Release archive signatures

- `RELEASE_GPG_PRIVATE_KEY_BASE64`: Base64-encoded armored private GPG key.
- `RELEASE_GPG_PASSPHRASE`: Private-key passphrase.
- `RELEASE_GPG_FINGERPRINT`: Full fingerprint without relying on a short key ID.

Generate the key with Young Hyun Chi's preferred public identity. The workflow verifies the imported fingerprint, exports the public key as `RELEASE_SIGNING_KEY.asc`, creates `SHA256SUMS`, and produces armored detached signatures for every archive and the checksum file. It also attaches GitHub build-provenance attestations to the release artifacts.

Verify a downloaded archive with:

```text
gpg --import RELEASE_SIGNING_KEY.asc
gpg --verify eu5-location-filter-v0.1.0-x86_64-pc-windows-msvc.zip.asc eu5-location-filter-v0.1.0-x86_64-pc-windows-msvc.zip
sha256sum --check SHA256SUMS
gh attestation verify eu5-location-filter-v0.1.0-x86_64-pc-windows-msvc.zip --repo younghyun1/eu5-location-filter
```

## Release gate

Update the Cargo version and changelog, commit to `main`, and create an annotated matching tag such as `v0.1.0`. Pushing that tag is the only release trigger. The workflow validates that the tag matches the Cargo version and points to a commit on `main`; no release is created when validation, signing, notarization, verification, checksums, signatures, or provenance attestation fails.
