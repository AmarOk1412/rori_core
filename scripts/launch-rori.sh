#!/bin/bash
export $(dbus-launch)
make keys
python3 scripts/generate_modules.py
make run
