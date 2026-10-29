#! /usr/bin/env bash
# Refresh test/conformance/shared from a committed CLASP-conformance revision.
#
# Reads with git archive only. Does not copy the source working tree, and
# does not run CLASP.
set -euo pipefail

ScriptDir=$(cd "$(dirname "$0")" && pwd)
CrateRoot=$(cd "$ScriptDir/.." && pwd)
SourceRepo=${CLASP_CONFORMANCE_REPO:-$(cd "$CrateRoot/../CLASP-conformance" && pwd)}
Revision=${1:-HEAD}
Dest="$CrateRoot/test/conformance/shared"

if [[ ! -d "$SourceRepo/.git" ]]; then
	echo "error: CLASP-conformance repository not found at $SourceRepo" >&2
	exit 1
fi

if ! Resolved=$(git -C "$SourceRepo" rev-parse --verify "${Revision}^{commit}"); then
	echo "error: revision not resolved: $Revision" >&2
	exit 1
fi

if [[ -n "$(git -C "$SourceRepo" status --porcelain)" ]]; then
	echo "warning: CLASP-conformance working tree is dirty; snapshot uses $Resolved, not the working tree" >&2
fi

Work=$(mktemp -d)
trap 'rm -rf "$Work"' EXIT

if ! git -C "$SourceRepo" archive "$Resolved" cases schema.md schema.json | tar -x -C "$Work"; then
	echo "error: git archive failed for $Resolved" >&2
	exit 1
fi

if [[ ! -f "$Work/schema.md" || ! -f "$Work/schema.json" || ! -d "$Work/cases" ]]; then
	echo "error: $Resolved does not contain cases/, schema.md, and schema.json" >&2
	exit 1
fi

mkdir -p "$CrateRoot/test/conformance"
rm -rf "$Dest"
mkdir -p "$Dest"
mv "$Work/cases" "$Dest/cases"
mv "$Work/schema.md" "$Dest/schema.md"
mv "$Work/schema.json" "$Dest/schema.json"

SnapshotDate=$(date +%Y-%m-%d)
{
	echo "source: $SourceRepo"
	echo "revision: $Resolved"
	echo "date: $SnapshotDate"
	echo "files:"
	(
		cd "$Dest"
		find . -type f ! -name MANIFEST | sed 's|^\./||' | LC_ALL=C sort
	)
} > "$Dest/MANIFEST"

echo "synced test/conformance/shared from $Resolved"
