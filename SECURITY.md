# Security policy

Please do not file public issues for suspected vulnerabilities. Report them privately to the maintainers with affected versions, reproduction steps, impact, and a safe proof of concept.

DiskScope treats filesystem paths as hostile input. Contributors must use native path APIs, must not interpolate a path into a shell command, and must preserve root-boundary and symlink checks before cleanup. Destructive tests must use fixtures only.
