# Release signing

Release artifacts are built and signed in CI (`.github/workflows/ci.yml`). The
desktop build (incl. macOS signing) runs **only on `vX.Y.Z` tags**, and the
macOS certificate lives in a **`release` GitHub Environment whose deployment
policy permits only `v*` tags** — so the Developer ID key is unreachable from
PRs, forks, or branch pushes.

| Platform | Mechanism | Status |
|----------|-----------|--------|
| Android  | Keystore in GH secrets (already set — **never regenerate**) | ✅ signed |
| macOS    | Developer ID Application cert + notarization (app **and** DMG) | ✅ signed |
| Windows  | Intentionally **unsigned** — SmartScreen "More info → Run anyway" | by choice (see below) |
| Linux    | none (AppImage/deb run unsigned) | n/a |

---

## macOS (Apple Developer, $99/yr)

Signed with a **Developer ID Application** certificate and notarized so the
downloaded DMG opens with no Gatekeeper warning. CI **auto-detects** the
Developer ID identity from the imported cert (there is no `APPLE_SIGNING_IDENTITY`
secret), notarizes + staples the `.app`, and then **also notarizes + staples the
`.dmg`** (Tauri only does the `.app`, so the DMG step is a separate `notarytool`
call — without it the downloaded DMG still trips Gatekeeper).

### Where the secrets live

The certificate is an **Environment secret** in the `release` environment
(Settings ▸ Environments ▸ release), which only `v*` tags may deploy to. The
non-key notarization creds are ordinary repo secrets.

| Secret | Scope | Value |
|--------|-------|-------|
| `APPLE_CERTIFICATE_BASE64` | **`release` env** | base64 of the Developer ID `.p12` (cert **+** private key) |
| `APPLE_CERTIFICATE_PASSWORD` | repo | the `.p12` export password |
| `APPLE_ID` | repo | your Apple ID email |
| `APPLE_APP_SPECIFIC_PASSWORD` | repo | app-specific password (account.apple.com ▸ Sign-In & Security) |
| `APPLE_TEAM_ID` | repo | Team ID (developer.apple.com ▸ Membership) |

### Rotating / re-exporting the cert

Export **only** the Developer ID identity: Keychain Access ▸ login ▸ **My
Certificates** ▸ expand the *Developer ID Application* entry ▸ select the cert
**and** its private key ▸ *Export 2 items* ▸ `.p12`. Then:

```bash
base64 -i ~/devid.p12 | gh secret set APPLE_CERTIFICATE_BASE64 --env release
gh secret set APPLE_CERTIFICATE_PASSWORD --env release --body "<that p12 password>"
```

> Gotchas learned the hard way: the secret must be a **Developer ID Application**
> cert (not *Apple Distribution*, which is App-Store-only); the export must
> include the **private key** (a cert-only `.p12` yields "0 valid identities");
> and `security import` must **not** use `-t cert` (that drops the key).

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
