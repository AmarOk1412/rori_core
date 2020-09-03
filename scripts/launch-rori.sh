#!/bin/bash
export $(dbus-launch)
python3 scripts/generate_modules.py
make run
