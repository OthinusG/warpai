# Warpai 2.0.0 Companion guardian

One Companion guardian per execution account keeps explicitly launched SSH
or WSL tasks alive across desktop close. A new terminal starts normally.
Use `warpai list`, `warpai resume task1` and `warpai stop task1` to manage the
existing process. Start a guarded task with `warpai run --name task1 -- ...`.

- [Requirements](PRD.md) and [technical design](TECH.md)
- [CLI contract](CLI.md) and [private API](API.md)
- [Implementation tasks](TASKS.md) and [verification](VERIFY.md)
- [Implementation results](IMPLEMENTATION.md) and [release plan](RELEASE.md)
