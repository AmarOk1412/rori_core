#!/bin/bash
export $(dbus-launch)
make keys
make run
