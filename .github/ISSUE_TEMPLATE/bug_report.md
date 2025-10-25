---
name: 🐛 Bug Report
about: Report a bug in WinnCore AV
title: '[BUG] '
labels: bug
assignees: ''
---

## Bug Description
A clear and concise description of what the bug is.

## Environment
**WinnCore Version:**
- Version: [e.g. v0.2.0]
- Installation method: [.deb / source build / install.sh]

**System:**
- OS: [e.g. Ubuntu 25.10]
- Architecture: [e.g. ARM64 / aarch64]
- Kernel: [output of `uname -r`]
- Hardware: [e.g. Raspberry Pi 4, ThinkPad X13s]

## To Reproduce
Steps to reproduce the behavior:
1. Run command '...'
2. Observe '...'
3. See error

## Expected Behavior
A clear description of what you expected to happen.

## Actual Behavior
What actually happened instead.

## Logs
<details>
<summary>Click to expand logs</summary>

```
Paste relevant logs here:
- journalctl -u av-daemon.service
- av-cli output
- /var/log/charmedwoa-av/
```

</details>

## Additional Context
- Does the issue occur with `sudo`?
- Any custom configuration changes?
- Recent system updates?
- Other AV software installed?

## Possible Solution (Optional)
If you have ideas on how to fix this, please share!
