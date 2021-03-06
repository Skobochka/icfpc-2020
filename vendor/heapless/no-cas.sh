#!/bin/bash

set -euo pipefail

main() {
    IFS='
'
    for t in $(rustc --print target-list); do
        rustc +nightly --print cfg --target $t | grep 'target_has_atomic=' >/dev/null || echo $t
    done

}

main
