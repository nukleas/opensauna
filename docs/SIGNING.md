# Release signing

Release artifacts are built and signed in CI (`.github/workflows/ci.yml`) on a
`vX.Y.Z` tag. Signing is **gated**: off-tag builds warn and ship unsigned (fine
for verification); on a tag, a missing signing secret **fails** the build so a
release is never published unsigned.

| Platform | Mechanism | Status |
|----------|-----------|--------|
| Android  | Keystore in GH secrets (already set — **never regenerate**) | ✅ signed |
| macOS    | Developer ID Application cert + notarization | ✅ signed (secrets below) |
| Windows  | Intentionally **unsigned** — SmartScreen "More info → Run anyway" | by choice (see below) |
| Linux    | none (AppImage/deb run unsigned) | n/a |

---

## macOS (Apple Developer, $99/yr)

Why: the app is currently ad-hoc signed (`signingIdentity: "-"`), so Gatekeeper
blocks it for anyone who downloads the DMG. A **Developer ID Application** cert
plus notarization makes it open with zero warnings on any Mac.

### One-time setup

1. **Create the certificate** — https://developer.apple.com/account/resources/certificates/list
   → **+** → **Developer ID Application** (only the Account Holder can create
   this). Generate a CSR from Keychain Access (*Certificate Assistant ▸ Request
   a Certificate From a Certificate Authority*, "Saved to disk"), upload it,
   download the `.cer`, double-click to install into your login keychain.
2. **Export as `.p12`** — in Keychain Access, find the cert, expand it, select
   **both** the cert and its private key → right-click → *Export 2 items* →
   `.p12`, set a password (you'll reuse it as `APPLE_CERTIFICATE_PASSWORD`).
3. **Base64-encode it** for the secret:
   ```bash
   base64 -i Certificates.p12 | pbcopy   # now paste into the secret
   ```
4. **App-specific password** — https://account.apple.com → Sign-In & Security ▸
   App-Specific Passwords ▸ generate one (this is `APPLE_PASSWORD`, **not** your
   Apple ID password).
5. **Team ID** — https://developer.apple.com/account → Membership details.
6. **Signing identity string** — exactly as shown by:
   ```bash
   security find-identity -v -p codesigning
   # e.g. "Developer ID Application: Your Name (TEAMID1234)"
   ```

### GitHub secrets (Settings ▸ Secrets and variables ▸ Actions ▸ Secrets)

| Secret | Value |
|--------|-------|
| `APPLE_CERTIFICATE_BASE64` | base64 of the `.p12` (step 3) |
| `APPLE_CERTIFICATE_PASSWORD` | the `.p12` export password (step 2) |
| `APPLE_SIGNING_IDENTITY` | the full identity string (step 6) |
| `APPLE_ID` | your Apple ID email |
| `APPLE_APP_SPECIFIC_PASSWORD` | app-specific password (step 4) |
| `APPLE_TEAM_ID` | Team ID (step 5) |

> Note: CI sets `APPLE_SIGNING_IDENTITY` as an env var to override the `"-"` in
> `tauri.conf.json`. If a signed build ever still comes out ad-hoc, remove the
> `signingIdentity` line from the config so the env var is the only source.

---

## Windows — intentionally unsigned

The Windows MSI/EXE ship **unsigned by choice.** The app runs fine; the only
difference is that on first launch Windows SmartScreen shows a blue dialog —
the user clicks **More info → Run anyway** once, and it's trusted from then on.

A real Authenticode signature (no warning) requires an identity-validated cert.
The cheapest path is **Azure Trusted Signing** (~$10/mo + an Azure subscription,
App Registration, and a government-ID identity validation). That overhead wasn't
worth it for this project. If that calculus changes, the setup is: create a
Trusted Signing account + Public Trust certificate profile, an Entra App
Registration with the **Trusted Signing Certificate Profile Signer** role, then
wire `cargo tauri build --config '{"bundle":{"windows":{"signCommand":"trusted-signing-cli -e <endpoint> -a <account> -c <profile> -d OpenSauna %1"}}}'`
with `AZURE_CLIENT_ID/SECRET/TENANT_ID` in CI.

### README note for users

> **Windows:** the installer is unsigned, so SmartScreen may warn on first run.
> Click **More info → Run anyway**. (macOS and Android builds are signed.)

---

## Releasing

The existing tag-driven flow signs macOS + Android automatically:

```bash
# bump version in src-tauri/tauri.conf.json + src-tauri/Cargo.toml first
git tag v0.5.0 && git push origin v0.5.0
```

CI builds **signed/notarized macOS** (DMG/.app), **signed Android** (universal
APK/AAB), **unsigned Windows** (MSI/EXE — SmartScreen one-click), and unsigned
Linux, then publishes the GitHub Release.
