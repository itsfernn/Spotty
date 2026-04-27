#!/bin/sh
    docker build --network=host -t spotty-doc .
    docker run --rm -it -e THEUID="$(id -u "$USER")" -v "$PWD":/var/doxerlive spotty-doc ash
