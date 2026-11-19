# Warpai 2.0.0 release verification

Reuse the platform packaging workflows. Include the CLI in the matching
Companion and WSL guest binaries. Test the installed manifest, native shell
entry, normal terminal fallback, and a compatible already-running service.
Publish only through the existing release gate after reviewing the same-source
SSH and WSL verification results. CI dispatch does not publish a release.

Document guarded `run`, explicit `resume`, detach, and exact `stop` in the
English and Chinese remote guides. Include the host-reboot and WSL-shutdown
lifecycle semantics.
