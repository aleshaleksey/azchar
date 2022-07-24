#!/bin/bash
azchar &
AZCHAR_PID=$!
third
kill -SIGKILL $AZCHAR_PID
