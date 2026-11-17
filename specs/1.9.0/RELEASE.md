# Warpai 1.9.0 release ledger

Date: 2026-11-17. Contract: [release preparation](../RELEASE.md).

## Scope

Desktop 1.9.0 / Companion 6.1.0. Preserve SSH major 1, Agent Bus major 2,
existing identity/recovery safeguards and published artifacts. All Rust/native
acceptance runs in GitHub Actions. No personal installation/configuration changes.

## Gates

- [ ] Website mobile header: bilingual controls, themes, drawer and desktop QA.
- [ ] Companion version/WSL packaging checks and disposable installer tests.
- [ ] Source-matched cloud protocol, application, native captures and real WSL.
- [ ] Three-platform Companion wire validation.
- [ ] Native screenshot and diagnostic review.
- [ ] Immutable v1.9.0 tag and draft macOS/Companion build, auto_publish=false.
- [ ] Windows x64 and Linux x64/ARM64 packages attached and verified.
- [ ] Thirteen installers, checksum/provenance inventories and public-release gate.
- [ ] Public download surfaces, production Pages and final Homebrew synchronization.

## Existing evidence

Main `1699828636cf125b43ecd46413918e5630f29984` passed communication
[44008543133](https://github.com/OthinusG/warpai/actions/runs/44008543133) and
Companion wire [44008543111](https://github.com/OthinusG/warpai/actions/runs/44008543111).
The push-triggered communication run did not request native captures or real WSL;
it cannot close those release gates or validate the pending 6.1.0 version change.
