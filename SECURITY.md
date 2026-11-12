# Security

## rind

This repository documents rind; it ships none of rind's code. Report a
vulnerability in rind to rind's maintainers privately, through GitHub's
**Report a vulnerability** button on
[moq-dev/rind's Security tab](https://github.com/moq-dev/rind/security),
not in an issue here or there.

## This repository

The only code here is the checker under `src/`, which is not published and
runs only in CI and in contributors' checkouts. If a skill tells readers to
do something unsafe (expose a secret, weaken a check, skip a constant-time
test), or a file here leaks something it shouldn't, report it privately
with **Report a vulnerability** on this repository's
[Security tab](https://github.com/steelhead99x/rind-agent/security).
Anything else is an ordinary issue.
