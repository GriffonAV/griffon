#!/bin/sh
set -e

# Create Griffon group used to access the daemon socket.
if ! getent group griffon >/dev/null 2>&1; then
    groupadd --system griffon
fi

# If the package is installed with sudo, add the original user to the griffon group.
if [ -n "$SUDO_USER" ] && [ "$SUDO_USER" != "root" ]; then
    usermod -aG griffon "$SUDO_USER" || true
    echo "User '$SUDO_USER' has been added to the griffon group."
fi

mkdir -p /usr/lib/griffon/plugins

if command -v systemctl >/dev/null 2>&1; then
    systemctl daemon-reload || true
    systemctl enable griffon-daemon || true
    systemctl restart griffon-daemon || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache /usr/share/icons/hicolor -f -t || true
fi

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database /usr/share/applications || true
fi

echo ""
echo "Griffon installed successfully."
echo ""
echo "The Griffon GUI and CLI require your user to be in the 'griffon' group"
echo "to communicate with the daemon without sudo."

if [ -n "$SUDO_USER" ] && [ "$SUDO_USER" != "root" ]; then
    echo ""
    echo "User '$SUDO_USER' has been added to the 'griffon' group."
    echo "You may need to log out and log back in for the change to apply."
else
    echo ""
    echo "If needed, add your user manually:"
    echo "  sudo usermod -aG griffon \$USER"
    echo ""
    echo "Then log out and log back in, or run:"
    echo "  newgrp griffon"
fi

echo ""