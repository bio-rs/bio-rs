#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
REPORT=
TMP="${TMPDIR:-/tmp}/biors-fixture-cross-checks.$$"
STATUS=0

usage() {
  echo "usage: $0 --write-report <path>" >&2
  exit 2
}

cleanup() {
  rm -rf "$TMP"
}
trap cleanup EXIT INT TERM

[ "${1:-}" = "--write-report" ] || usage
[ -n "${2:-}" ] || usage
REPORT=$2
mkdir -p "$TMP" "$(dirname -- "$REPORT")"
: > "$TMP/report.md"

note() {
  printf '%s\n' "$*" >> "$TMP/report.md"
}

tool_version() {
  if command -v "$1" >/dev/null 2>&1; then
    "$@" 2>&1 | head -n 1
  else
    printf 'not available'
  fi
}

check_sequence() {
  protein="$ROOT/testdata/researcher-workflows/protein.fasta"
  dna="$ROOT/testdata/researcher-workflows/dna.fasta"
  rna="$ROOT/testdata/researcher-workflows/rna.fasta"
  fastq="$ROOT/testdata/researcher-workflows/reads.fastq"

  note "## Sequence Fixtures"
  if command -v seqkit >/dev/null 2>&1; then
    note "- tool: seqkit ($(tool_version seqkit version))"
    if seqkit stats -T "$protein" "$dna" "$rna" "$fastq" > "$TMP/seqkit.tsv"; then
      note "- result: PASS - seqkit parsed protein, DNA, RNA, and FASTQ fixtures."
      note "- artifact: seqkit.tsv"
    else
      note "- result: FAIL - seqkit could not parse one or more sequence fixtures."
      STATUS=1
    fi
  elif python3 - <<'PY' >/dev/null 2>&1
import Bio.SeqIO
PY
  then
    note "- tool: Biopython ($(python3 - <<'PY'
import Bio
print(Bio.__version__)
PY
))"
    python3 - "$protein" "$dna" "$rna" "$fastq" > "$TMP/biopython-sequence.txt" <<'PY' || STATUS=1
import sys
from Bio import SeqIO

protein, dna, rna, fastq = sys.argv[1:]
checks = [
    ("protein", protein, "fasta", 1),
    ("dna", dna, "fasta", 1),
    ("rna", rna, "fasta", 1),
    ("fastq", fastq, "fastq", 1),
]
for name, path, fmt, expected in checks:
    records = list(SeqIO.parse(path, fmt))
    if len(records) != expected:
        raise SystemExit(f"{name}: expected {expected}, got {len(records)}")
    print(f"{name}: PASS records={len(records)}")
PY
    if [ "$STATUS" -eq 0 ]; then
      note "- result: PASS - Biopython parsed protein, DNA, RNA, and FASTQ fixtures."
      note "- artifact: biopython-sequence.txt"
    else
      note "- result: FAIL - Biopython sequence count mismatch."
    fi
  else
    note "- tool: SKIP - seqkit and Biopython are unavailable."
    note "- result: SKIP - sequence external parser checks are not 1.0-ready evidence."
  fi
  note
}

check_molecule() {
  smiles="$ROOT/testdata/researcher-workflows/molecule.smi"

  note "## Molecule Fixtures"
  if python3 - <<'PY' >/dev/null 2>&1
from rdkit import Chem
PY
  then
    note "- tool: RDKit ($(python3 - <<'PY'
import rdkit
print(rdkit.__version__)
PY
))"
    python3 - "$smiles" > "$TMP/rdkit-molecule.txt" <<'PY' || STATUS=1
import sys
from rdkit import Chem

path = sys.argv[1]
with open(path, encoding="utf-8") as handle:
    token = handle.readline().split()[0]
mol = Chem.MolFromSmiles(token)
if mol is None:
    raise SystemExit("RDKit failed to parse SMILES")
print(f"RDKit PASS atoms={mol.GetNumAtoms()}")
PY
    if [ "$STATUS" -eq 0 ]; then
      note "- result: PASS - RDKit parsed the SMILES fixture."
      note "- artifact: rdkit-molecule.txt"
    else
      note "- result: FAIL - RDKit failed the SMILES sanity check."
    fi
  elif command -v obabel >/dev/null 2>&1; then
    note "- tool: Open Babel ($(tool_version obabel -V))"
    if obabel -ismi "$smiles" -ocan -O "$TMP/openbabel.can" >/dev/null 2>&1 && [ -s "$TMP/openbabel.can" ]; then
      note "- result: PASS - Open Babel canonicalized the SMILES fixture."
      note "- artifact: openbabel.can"
    else
      note "- result: FAIL - Open Babel failed the SMILES fixture."
      STATUS=1
    fi
  else
    note "- tool: SKIP - RDKit and Open Babel are unavailable."
    note "- result: SKIP - molecule external parser checks are not 1.0-ready evidence."
  fi
  note
}

check_structure() {
  pdb="$ROOT/testdata/researcher-workflows/structure.pdb"

  note "## Structure Fixtures"
  if python3 - <<'PY' >/dev/null 2>&1
import Bio.PDB
PY
  then
    note "- tool: Biopython Bio.PDB ($(python3 - <<'PY'
import Bio
print(Bio.__version__)
PY
))"
    python3 - "$pdb" > "$TMP/biopython-pdb.txt" <<'PY' || STATUS=1
import sys
from Bio.PDB import PDBParser

parser = PDBParser(QUIET=True)
structure = parser.get_structure("fixture", sys.argv[1])
atoms = list(structure.get_atoms())
if len(atoms) != 3:
    raise SystemExit(f"expected 3 atoms, got {len(atoms)}")
print(f"Bio.PDB PASS atoms={len(atoms)}")
PY
    if [ "$STATUS" -eq 0 ]; then
      note "- result: PASS - Bio.PDB parsed the PDB fixture."
      note "- artifact: biopython-pdb.txt"
    else
      note "- result: FAIL - Bio.PDB atom-count mismatch."
    fi
  elif python3 - <<'PY' >/dev/null 2>&1
import gemmi
PY
  then
    note "- tool: gemmi ($(python3 - <<'PY'
import gemmi
print(gemmi.__version__)
PY
))"
    python3 - "$pdb" > "$TMP/gemmi-pdb.txt" <<'PY' || STATUS=1
import sys
import gemmi

structure = gemmi.read_structure(sys.argv[1])
atoms = sum(1 for model in structure for chain in model for residue in chain for atom in residue)
if atoms != 3:
    raise SystemExit(f"expected 3 atoms, got {atoms}")
print(f"gemmi PASS atoms={atoms}")
PY
    if [ "$STATUS" -eq 0 ]; then
      note "- result: PASS - gemmi parsed the PDB fixture."
      note "- artifact: gemmi-pdb.txt"
    else
      note "- result: FAIL - gemmi atom-count mismatch."
    fi
  else
    awk '/^ATOM  / { count += 1 } END { if (count != 3) exit 1 }' "$pdb" || STATUS=1
    note "- tool: SKIP - Bio.PDB and gemmi are unavailable."
    note "- result: SKIP - only internal PDB ATOM count sanity was possible; structure external parser checks are not 1.0-ready evidence."
  fi
  note
}

{
  note "# 1.0 Parser Cross-Check Audit"
  note
  note "Generated by \`scripts/check-fixture-cross-checks.sh --write-report $REPORT\`."
  note "External tools are optional. Unavailable tools are recorded as SKIP, and skipped areas must not be used as evidence for a 1.0 ready verdict."
  note
  check_sequence
  check_molecule
  check_structure
}

mv "$TMP/report.md" "$REPORT"
exit "$STATUS"
