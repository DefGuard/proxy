#!/bin/sh
CONFIG=/etc/defguard/proxy.toml

if [ ! -f "${CONFIG}" ]; then
    cp "${CONFIG}.sample" "${CONFIG}"
fi
