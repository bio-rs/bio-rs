import {
  browserExecutionPolicy,
  inspectBrowserFile,
  validateBrowserFile,
  tokenizeBrowserFile,
} from "../../packages/rust/biors-wasm/pkg/biors_wasm.js";

const fileInput = document.querySelector("#file");
const formatInput = document.querySelector("#format");
const kindInput = document.querySelector("#kind");
const profileInput = document.querySelector("#profile");
const output = document.querySelector("#output");

document.querySelector("#validate").addEventListener("click", async () => {
  await run("validate");
});

document.querySelector("#tokenize").addEventListener("click", async () => {
  await run("tokenize");
});

async function run(mode) {
  try {
    const input = await readBrowserInput();
    const policy = browserExecutionPolicy();
    const inspected = inspectBrowserFile(input);
    const result =
      mode === "tokenize" ? tokenizeBrowserFile(input) : validateBrowserFile(input);

    output.textContent = JSON.stringify({ policy, inspected, result }, null, 2);
  } catch (error) {
    output.textContent = error instanceof Error ? error.message : String(error);
  }
}

async function readBrowserInput() {
  const file = fileInput.files?.[0];
  if (!file) {
    throw new Error("Choose a local file first.");
  }

  const bytes = new Uint8Array(await file.arrayBuffer());
  const input = {
    name: file.name,
    bytes,
    kind: kindInput.value,
    profile: profileInput.value,
  };

  if (formatInput.value) {
    input.format = formatInput.value;
  }

  return input;
}
