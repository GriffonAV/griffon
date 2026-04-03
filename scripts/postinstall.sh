#!/bin/bash
systemctl daemon-reload
systemctl enable griffonav-daemon
systemctl start griffonav-daemon