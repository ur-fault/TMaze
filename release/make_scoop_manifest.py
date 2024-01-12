#!/bin/python3
from subprocess import run as run_process
from json import loads, dumps
from pprint import pprint
from hashlib import sha256
from urllib import request

metadata = loads(
    run_process(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"], capture_output=True
    ).stdout.decode("utf-8")
)

tmaze_metadata = metadata["packages"][0]

version = tmaze_metadata["version"]
desc = tmaze_metadata["description"]

tmaze_url = "https://github.com/ur-fault/tmaze"

release_assets = f"{tmaze_url}/releases/download/{version}"

download_url = f"{release_assets}/tmaze-{version}-win-x86_64.exe#/tmaze.exe"
scoop_manifest = f"{release_assets}/tmaze.json"

manifest = {
    "version": version,
    "architecture": {
        "64bit": {
            "url": download_url,
        },
        "checkver": {
            "url": "{scoop_manifest}",
            "jsonpath": "$.version",
        },
        "autoupdate": {},
    },
    "description": desc,
    "homepage": tmaze_url,
}

print(dumps(manifest, indent=4))
