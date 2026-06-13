# Release signing

Release artifacts are built and signed in CI (`.github/workflows/ci.yml`) on a
`vX.Y.Z` tag. Signing is **gated**: off-tag builds warn and ship unsigned (fine
for verification); on a tag, a missing signing secret **fails** the build so a
release is never published unsigned.

| Platform | Mechanism | Required to "run right" |
|----------|-----------|--------------------------|
| Android  | Keystore in GH secrets (already set — **never regenerate**) | ✅ done |
| macOS    | Developer ID Application cert + notarization | secrets below |
| Windows  | Azure Trusted Signing | secrets + repo vars below |
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

## Windows (Azure Trusted Signing, ~$10/mo)

Why: the MSI/EXE are unsigned, so SmartScreen warns on every download. Azure
Trusted Signing is the cheapest cert that gives real signatures.

### One-time setup

1. In the Azure portal, create a **Trusted Signing account** and a
   **Certificate Profile** (identity-validated). Note the **endpoint** region
   (e.g. `https://wus2.codesigning.azure.net`), the **account name**, and the
   **profile name**.
2. Create an **App Registration** (Entra ID) with a client secret, and grant it
   the **Trusted Signing Certificate Profile Signer** role on the account.

### GitHub secrets

| Secret | Value |
|--------|-------|
| `AZURE_CLIENT_ID` | App Registration application (client) ID |
| `AZURE_CLIENT_SECRET` | client secret |
| `AZURE_TENANT_ID` | directory (tenant) ID |

### GitHub repo Variables (same page ▸ Variables tab — not secret)

| Variable | Value |
|----------|-------|
| `AZURE_ENDPOINT` | e.g. `https://wus2.codesigning.azure.net` |
| `AZURE_ACCOUNT` | your Trusted Signing account name |
| `AZURE_PROFILE` | your certificate profile name |

CI installs `trusted-signing-cli` and passes a `signCommand` to Tauri only when
`AZURE_CLIENT_ID` is present.

---

## Releasing

Once secrets/vars are in place, the existing tag-driven flow signs everything:

```bash
# bump version in src-tauri/tauri.conf.json + src-tauri/Cargo.toml first
git tag v0.5.0 && git push origin v0.5.0
```

CI builds signed macOS (notarized DMG/.app), signed Windows (MSI/EXE), signed
Android (universal APK/AAB), and unsigned Linux, then publishes the GitHub
Release.
