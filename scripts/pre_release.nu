#!/bin/nu

# Run this script before releasing a new version, in creating or merging a pull request.
# In case, this script is run as a pre-release, enable --versions flag to check if the version number in the project file is correct.

# This script should be run from the root of the project directory.

# This script will:
# - Check that everything compiles
# - Run the tests
# - Check if the version number in the project file is correct
#
# Required dependencies:
# - cargo
# - jq
# - nu
# - toml-cli
# - semver-cli

def compareVersions [a: record, b: record] -> int {
    if ($a.major != $b.major) {
        return ($a.major - $b.major)
    }

    if ($a.minor != $b.minor) {
        return ($a.minor - $b.minor)
    }

    return ($a.patch - $b.patch)
}

def checkVersion [package: string] {
    print $"- ($package)..."

    let this = (toml get $"./($package)/Cargo.toml" package.version -r)
    let released = (cargo search $package --limit 1 | grep -E '[0-9]+\\.[0-9]+\\.[0-9]+' -o)
    
    let this = semver $this | from json
    let released = semver $released | from json

    if ((compareVersions $this $released) <= 0) {
        print $"Please update the version number in ($package)/Cargo.toml"
        print $"Current version: ($this.version), newest released version: ($released.version)"
    }
}

def main [
    --versions: bool # Enable version checking
] {
    print "Running tests..."
    cargo test

    print "\nBuilding cmaze..."
    cargo build -p cmaze

    print "\nBuilding tmaze..."
    cargo build -p tmaze

    print "\nBuilding compress..."
    cargo build -p compress

    if ($versions) {
        print "\nChecking version numbers..."
        checkVersion "cmaze"
        checkVersion "tmaze"
        # checkVersion "compress" # compress is not on crates.io
    } else {
        print "\nSkipping version number check..."
    }
}
