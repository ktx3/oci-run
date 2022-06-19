# This is free and unencumbered software released into the public domain.

# Helper script for oci-run aliased commands
#
# Create symlinks to this script in your PATH to create aliases for commands
# that should be executed using oci-run.
#
# Optional environment variables:
#
# - OCI_RUN_OPTS: options passed to oci-run

set -e -u

exec oci-run ${OCI_RUN_OPTS:-} -- "${0##*/}" "${@}"
