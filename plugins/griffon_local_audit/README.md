# Griffon Local Security Audit

This first-party plugin performs a bounded, read-only audit of local Linux security
posture. It is inspired by the defensive inventory categories exposed by the
[PEASS-ng linPEAS builder](https://github.com/peass-ng/PEASS-ng/tree/master/linPEAS/builder),
but its implementation is original Rust code. Griffon does not embed or execute
linPEAS.

The plugin reads procfs, sysfs and selected local configuration metadata directly.
It does not invoke a shell or external command, connect to discovered sockets,
probe the network, change configuration, or attempt privilege escalation. Results
are evidence, not proof that a system is secure or compromised.

## Defensive checks

- System and protection inventory: operating system, kernel, architecture,
  container markers, active Linux security modules, SELinux, AppArmor, kernel
  lockdown, Secure Boot and selected kernel taint flags.
- Kernel and network policy: link and FIFO protections, unprivileged eBPF,
  performance events, low-address mappings, core dumps, SYN cookies, reverse-path
  filtering, redirects, source routing and IP forwarding.
- Accounts and authentication: duplicate UIDs, interactive accounts, home and SSH
  path ownership, empty password fields, password-aging defaults, hashing policy,
  umask and PAM lockout configuration presence.
- Privileged access: root-equivalent identities, security-sensitive local groups,
  SUID/SGID executables and Linux file capabilities in standard binary directories.
- Runtime integrity: bounded process metadata, process capabilities, root process
  executable ownership and writability, deleted executables, systemd service and
  drop-in permissions, service executable paths and Unix socket metadata.
- Filesystem and policy integrity: sensitive files, system policy directories,
  scheduled-task paths, temporary mount options, network filesystems, NFS exports,
  daemon search-path entries, dynamic-linker configuration and preload targets.
- Service exposure and evidence retention: direct SSH daemon directives, listening
  TCP ports, bound UDP ports, local Unix sockets, firewall configuration presence,
  security-log permissions, journald storage and pending-reboot markers.

Actionable findings and informational inventory entries are returned separately.
An expected listening port, privileged executable, account or security module is
therefore not presented as a defect by itself.

## Deliberate exclusions

Offensive or unnecessarily sensitive linPEAS-style techniques are outside this
plugin's scope. It does not brute-force `su`, search files or process environments
for passwords, read clipboard contents, query cloud metadata services, interact
with sockets, map findings to exploitation recipes, suggest exploits, or perform
network discovery. It also avoids reporting password hashes, command lines and
environment variables.

## Bounds and failure handling

Text files, account lists, process lists, unit files, policy directories, EFI
variables and executable walks have fixed limits. Symbolic links are not followed
during executable inventory, and the SUID/SGID/file-capability walk stops after a
fixed number of filesystem entries. Report limits supplied by the caller are
clamped by the plugin. Read failures and unsupported checks are returned as
diagnostics instead of being treated as successful checks.
