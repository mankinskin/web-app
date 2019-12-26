#!/bin/bash
kill $(pidof simple-http-server); \
simple-http-server & \
