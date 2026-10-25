#!/bin/sh
set -e
printf 'def total(cents):\n    return sum(cents)\n' > cart.py
celis decide --situation -- "When a refund is computed: round it down to the cent, because the shop never pays back more than it took." >/dev/null
