# Testing the packages

Two levels of testing:

| What | Where | Catches |
|------|-------|---------|
| Install test (automatic) | `Griffon Packaging` workflow, on every push to `main` and before every release | Missing/wrong dependencies, glibc too new, broken install scripts |
| Full test in a VM (manual) | Vagrant, `packaging/test/` | Everything above **plus** the systemd daemon, the `griffon` group / socket rights, the CLI and GUI |

Containers have no systemd, so the daemon and the rights can only be checked in
a VM. Do the VM test before tagging a release.

## Supported systems

The binaries are built on Debian 13 and need **glibc ≥ 2.39**:
Ubuntu 24.04+, Debian 13+, Fedora 40+. Ubuntu 22.04 / Debian 12 are **not**
supported (the daemon fails with `GLIBC_2.39 not found`).

## 1. Build the packages

```bash
docker build -t griffon-builder -f Dockerfile.build .
docker run --rm -e VERSION=0.0.0 -v "$(pwd)/dist:/out" griffon-builder
ls dist/   # griffon-0.0.0.deb, griffon-0.0.0.rpm
```

## 2. Create a clean VM (once)

Requirements: Vagrant (HashiCorp) and VirtualBox.

```bash
cd packaging/test
vagrant up                      # Ubuntu 24.04 ("vagrant up fedora" for the RPM)
vagrant ssh -c "sudo apt update && sudo apt upgrade -y"
vagrant snapshot save clean
```

`dist/` is mounted at `/dist` in the VM: rebuilt packages show up without
restarting it.

> If `vagrant up` complains about KVM: `sudo modprobe -r kvm_intel kvm`
> (VirtualBox and KVM can't both use the CPU virtualization).

## 3. Install and test as a user

```bash
vagrant ssh
sudo apt install -y /dist/griffon-*.deb       # Fedora: sudo dnf install -y /dist/griffon-*.rpm
systemctl status griffon-daemon               # must be "active (running)"
```

The installer adds the user who ran `sudo` to the `griffon` group, but this
only takes effect after a new login:

```bash
griffon-cli ...      # permission denied is expected here
exit
vagrant ssh          # log in again
id                   # 'griffon' must be listed
griffon-cli ...      # must work
```

If the daemon doesn't start:

```bash
journalctl -u griffon-daemon -b --no-pager | tail -20
```

## 4. Reset

```bash
vagrant snapshot restore clean   # back to a clean system
vagrant destroy -f               # delete the VM
```

## Known limitation: user rights

Users must be in the `griffon` group to talk to the daemon. The release notes
must say:

> After installing, log out and back in (or run `newgrp griffon`). If you
> installed without `sudo` (e.g. as root), add your user manually:
> `sudo usermod -aG griffon $USER`.
