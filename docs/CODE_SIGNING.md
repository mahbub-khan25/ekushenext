# Code signing (Azure Artifact Signing)

When the program is signed, Windows shows the publisher's name where it used to say "Unknown
publisher", and SmartScreen and Defender can build trust for it over time. Signing happens
automatically in `.github/workflows/release.yml` once the settings below exist. Until then,
releases are published unsigned.

Cost: Basic plan, US$9.99/month (5,000 signatures). Individuals in the USA and Canada can apply.
One account and certificate profile sign all your apps: Marrow uses the same ones (see its
`RELEASE.md`); each repository only needs its own federated credential and the same six settings.

## 1. Azure account and signing account

1. Sign in at <https://portal.azure.com>. Creating an account needs a credit card.
2. Search for **Artifact Signing**. Older pages call it **Trusted Signing**. Click **Create**.
   - Subscription: your subscription. Resource group: create one, e.g. `ekushe-signing`.
   - Account name: e.g. `ekushe`. This becomes `SIGNING_ACCOUNT`.
   - Region: e.g. **East US**. The region sets the endpoint, which becomes `SIGNING_ENDPOINT`:

     | Region          | Endpoint                              |
     |-----------------|---------------------------------------|
     | East US         | `https://eus.codesigning.azure.net/`  |
     | West US         | `https://wus.codesigning.azure.net/`  |
     | West Central US | `https://wcus.codesigning.azure.net/` |
     | North Europe    | `https://neu.codesigning.azure.net/`  |
     | West Europe     | `https://weu.codesigning.azure.net/`  |

   - Plan: **Basic**.
3. Give yourself permission to request identity validation. In the signing account, go to
   **Access control (IAM) → Add role assignment** and assign **Artifact Signing Identity Verifier**
   (older name: *Trusted Signing Identity Verifier*) to your own user.

## 2. Identity validation (you, once)

In the signing account, open **Identity validation → New identity → Public → Individual**.
Fill in your legal name and address exactly as they appear on your government photo ID, then
follow the verification link Microsoft emails you. Approval takes from a few hours to a few days.
The name you validate is the publisher name Windows will show.

## 3. Certificate profile

After validation is approved, open **Certificate profiles → Create → Public Trust**. Pick the
approved identity and a profile name, e.g. `ekushe-public`. This becomes `SIGNING_PROFILE`.

## 4. Let GitHub sign without a stored password (OIDC)

1. In the Azure portal, open **Microsoft Entra ID → App registrations → New registration**.
   Name it `ekushe-github-signing` and leave the defaults.
   - Copy the **Application (client) ID**. This becomes `AZURE_CLIENT_ID`.
   - Copy the **Directory (tenant) ID**. This becomes `AZURE_TENANT_ID`.
2. In that app, go to **Certificates & secrets → Federated credentials → Add credential →
   GitHub Actions deploying Azure resources**:
   - Organization: `mahbub-khan25`, Repository: `ekushenext`
   - Entity type: **Branch**, Branch: `main`
   - Name: `release-main`
   - If you also publish releases by pushing tags, add a second credential with entity type
     **Tag** and tag `v*`.
3. Back in the signing account, open the **certificate profile → Access control (IAM) → Add
   role assignment**. Choose **Artifact Signing Certificate Profile Signer** (older name:
   *Trusted Signing Certificate Profile Signer*) and assign it to the app `ekushe-github-signing`.
4. Copy your **Subscription ID** from *Subscriptions*. This becomes `AZURE_SUBSCRIPTION_ID`.

## 5. Add the settings to GitHub

Go to repository **Settings → Secrets and variables → Actions**.

**Secrets** tab:

| Name                    | Value                             |
|-------------------------|-----------------------------------|
| `AZURE_CLIENT_ID`       | Application (client) ID           |
| `AZURE_TENANT_ID`       | Directory (tenant) ID             |
| `AZURE_SUBSCRIPTION_ID` | Subscription ID                   |

**Variables** tab:

| Name               | Example                               |
|--------------------|---------------------------------------|
| `SIGNING_ACCOUNT`  | `ekushe`                              |
| `SIGNING_PROFILE`  | `ekushe-public`                       |
| `SIGNING_ENDPOINT` | `https://eus.codesigning.azure.net/`  |

The next release signs `ekushe.exe`, `ekushe_tsf.dll` and `ekushe-cli.exe`, then checks that each
signature is valid before publishing. To publish a release, bump `.github/release/VERSION` and push.

## Notes

- Certificates last one day and renew automatically. Signatures are time-stamped, so signed
  releases stay valid.
- These are not EV certificates. SmartScreen reputation still builds up as people download and
  run the app, so the first signed releases may still show a warning for a while.
- Defender's malware check is separate. If a signed build is still flagged, report it at
  <https://www.microsoft.com/wdsi/filesubmission> ("Incorrectly detected as malware").
