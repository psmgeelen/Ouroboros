# Ouroboros

Ouroboros is a wellness app for iOS and Android that helps someone keep a private record of their menstrual cycle. Predictions are statistical estimates from that person's own logs. Ouroboros is not a contraceptive and it is not a medical device.

The tracker phone holds the full history. A partner can see only a narrow, current view, and only while the tracker allows it. The relay stores ciphertext it cannot read.

## Local setup

Install [Task](https://taskfile.dev/) and [pre-commit](https://pre-commit.com/), then:

```bash
task setup
task ci
```

`task setup` installs the git hooks and commitlint. Those hooks already cover whitespace, YAML, Rustfmt, Terraform fmt, and Dart format. `task ci` runs the crate, relay, mobile, and infra checks in parallel and does not repeat the whitespace hooks.

Commits use [Conventional Commits](https://www.conventionalcommits.org/). A message such as `update stuff` is rejected. A message such as `chore: scaffold the ouroboros repository` is accepted.
