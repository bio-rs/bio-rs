const crypto = require("crypto");
const fs = require("fs");
const wasm = require(process.argv[2]);
const fasta = fs.readFileSync(process.argv[3]);
const dnaFasta = fs.readFileSync(process.argv[4]);
const rnaFasta = fs.readFileSync(process.argv[5]);
const loops = Number(process.argv[6]);
const input = JSON.parse(process.argv[7]);
const dnaInput = JSON.parse(process.argv[8]);
const rnaInput = JSON.parse(process.argv[9]);

const parsedRecords = wasm.parseFasta(fasta);
const tokenizedRecords = wasm.tokenize(parsedRecords, "protein-20");
const parsedDnaRecords = wasm.parseFasta(dnaFasta);
const parsedRnaRecords = wasm.parseFasta(rnaFasta);
const workflowConfig = {
  fastaBytes: Uint8Array.from(fasta),
  maxLength: 160,
  padding: "fixed_length",
  padTokenId: 0,
};
const dnaWorkflowConfig = {
  fastaBytes: Uint8Array.from(dnaFasta),
  kind: "dna",
  profile: "dna-iupac",
  maxLength: 160,
  padding: "fixed_length",
  padTokenId: 0,
};
const rnaWorkflowConfig = {
  fastaBytes: Uint8Array.from(rnaFasta),
  kind: "rna",
  profile: "rna-iupac",
  maxLength: 160,
  padding: "fixed_length",
  padTokenId: 0,
};

function hash(value) {
  return `sha256:${crypto.createHash("sha256").update(JSON.stringify(value)).digest("hex")}`;
}

function timed(name, fn, workloadInput = input) {
  fn();
  const seconds = [];
  let output;
  for (let index = 0; index < loops; index += 1) {
    const start = process.hrtime.bigint();
    output = fn();
    seconds.push(Number(process.hrtime.bigint() - start) / 1e9);
  }
  seconds.sort((a, b) => a - b);
  const mean = seconds.reduce((sum, value) => sum + value, 0) / seconds.length;
  return {
    name,
    surface: "wasm_bindings",
    input: workloadInput,
    summary: {
      mean_s: mean,
      median_s: seconds[Math.floor(seconds.length / 2)],
      min_s: seconds[0],
      max_s: seconds[seconds.length - 1],
      output_hash: hash(output),
      output_bytes: Buffer.byteLength(JSON.stringify(output)),
    },
  };
}

process.stdout.write(JSON.stringify([
  timed("wasm_parse_fasta", () => wasm.parseFasta(fasta)),
  timed("wasm_validate_fasta", () => wasm.validateFasta(fasta, "protein")),
  timed("wasm_tokenize", () => wasm.tokenize(parsedRecords, "protein-20")),
  timed("wasm_run_workflow", () => wasm.runWorkflow(workflowConfig)),
  timed("wasm_tokenize_dna_iupac", () => wasm.tokenize(parsedDnaRecords, "dna-iupac"), dnaInput),
  timed("wasm_run_workflow_dna_iupac", () => wasm.runWorkflow(dnaWorkflowConfig), dnaInput),
  timed("wasm_tokenize_rna_iupac", () => wasm.tokenize(parsedRnaRecords, "rna-iupac"), rnaInput),
  timed("wasm_run_workflow_rna_iupac", () => wasm.runWorkflow(rnaWorkflowConfig), rnaInput),
]));
