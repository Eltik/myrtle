#!/usr/bin/env bash
# Fetch the embedder and the reranker at pinned revisions and verify every
# file by sha256. The gte tokenizer.json hash is the one chunks.jsonl is
# counted with; embed-corpus refuses to run if the two differ.
#
#   ./scripts/fetch-model.sh            # into ./models
#   ./scripts/fetch-model.sh DIR        # into DIR
#   WITH_FP32=1 ./scripts/fetch-model.sh
#       also the fp32 gte export (596 MB), for embedding queries on x86,
#       where the INT8 export drifts to cosine 0.94 from fp32.
#   WITH_LLM=1 ./scripts/fetch-model.sh
#       also the two local LLMs for gold-set generation (12.7 GB): Gemma 4
#       12B QAT Q4_0 writes questions, Qwen3.5 9B Q4_K_M judges them. Two
#       model families on purpose, so the judge is not grading its own work.
#   WITH_VISION=1 ./scripts/fetch-model.sh
#       also Gemma 4 12B's multimodal projector (175 MB) for image input.
#   WITH_E2B=1 ./scripts/fetch-model.sh
#       also Gemma 4 E2B instruct Q4_K_M (3.1 GB), the VPS CPU rung's model
#       (the slim profile; design/experiment-log.md, serving).
set -euo pipefail

ROOT="${1:-models}"

sha() { if command -v sha256sum >/dev/null; then sha256sum "$1"; else shasum -a 256 "$1"; fi | cut -d' ' -f1; }

# fetch REPO REV DIR "remote|local|sha256"...
fetch() {
  local repo="$1" rev="$2" dir="$3"
  shift 3
  local row remote name want out got
  for row in "$@"; do
    IFS='|' read -r remote name want <<<"$row"
    out="$dir/$name"
    mkdir -p "$(dirname "$out")"
    if [[ -f "$out" && "$(sha "$out")" == "$want" ]]; then
      echo "ok (cached) $out"
      continue
    fi
    trap 'rm -f "$out.part"' EXIT
    curl -fL --retry 3 -o "$out.part" "https://huggingface.co/$repo/resolve/$rev/$remote"
    got="$(sha "$out.part")"
    if [[ "$got" != "$want" ]]; then
      rm -f "$out.part"
      echo "sha256 mismatch for $out: got $got, want $want" >&2
      exit 1
    fi
    mv "$out.part" "$out"
    trap - EXIT
    echo "ok $out"
  done
}

GTE=(
  "tokenizer.json|tokenizer.json|6c8aaa9a542084f2457eab775d4eeb51f92a70c0fd9de28d5edb0ddec3c08d30"
  "tokenizer_config.json|tokenizer_config.json|9654072f7c873161814043cf08cb5ed72f71d0b935abcd4e267935cb34352c21"
  "special_tokens_map.json|special_tokens_map.json|ea97ecdbcc73713039d8d64dbb05e3689495c96657fbd9a18f5bed381be81049"
  "config.json|config.json|8ba54dc3d35d7194f5178a4194b649f146753e02dabd22bdca5c5cbac15069ed"
  "onnx/model_int8.onnx|model_int8.onnx|bae96b276d342bf86eeee07c1bdbc0c75bb82bf4033941aab7fabc1e33ee3b44"
)
if [[ "${WITH_FP32:-0}" == "1" ]]; then
  GTE+=("onnx/model.onnx|model.onnx|947f31df7effaeec4edb57c50e4ed7e0f2034d9336063f92615b92e3e0d24d78")
fi
# main as of 2025-07-04
fetch Alibaba-NLP/gte-modernbert-base e7f32e3c00f91d699e8c43b53106206bcc72bb22 \
  "$ROOT/gte-modernbert-base" "${GTE[@]}"

# The ONNX file is the backbone only; the scoring head is the three
# safetensors files, applied in src/search/rerank.rs.
fetch cross-encoder/ettin-reranker-17m-v1 9e4aa35321a6dd1a43ca313f500c4b4f7cfb5cc6 \
  "$ROOT/ettin-reranker-17m-v1" \
  "onnx/model.onnx|onnx/model.onnx|2a375ccea150a4df925a47884b510eeb4bd6227ca6fffdbbc62ca2284bfe6211" \
  "tokenizer.json|tokenizer.json|28c5e078e4c52aa37cf0e6de1a212878f3dbd58dd1c70466298efe0b6b86db35" \
  "config.json|config.json|f83c222e8684701525bfa5e0aaa2371724974dc7fb728174cca02e280feda1ad" \
  "2_Dense/model.safetensors|2_Dense/model.safetensors|85e9596d9250a871deb159fb5db6979e910b4cf181d05c806733c49bc43d47c8" \
  "3_LayerNorm/model.safetensors|3_LayerNorm/model.safetensors|de99fa351fb4badb74b56e85fa70b5bbd3fcf4d0e74de79eb749dba1e9e28b4a" \
  "4_Dense/model.safetensors|4_Dense/model.safetensors|654827171b89c76d19d663162243f38d63d1ba812ac1ec9c1b36512f1a8e9ce8"

if [[ "${WITH_LLM:-0}" == "1" ]]; then
  fetch google/gemma-4-12B-it-qat-q4_0-gguf 29d097773436b69ff9feafd636ab4cf873786537 "$ROOT/llm" \
    "gemma-4-12b-it-qat-q4_0.gguf|gemma-4-12b-it-qat-q4_0.gguf|93567e57a8fe10b23569b9d9ec38cd005deedf71e29477c421a4b83f418a538b"
  fetch unsloth/Qwen3.5-9B-GGUF 3885219b6810b007914f3a7950a8d1b469d598a5 "$ROOT/llm" \
    "Qwen3.5-9B-Q4_K_M.gguf|Qwen3.5-9B-Q4_K_M.gguf|03b74727a860a56338e042c4420bb3f04b2fec5734175f4cb9fa853daf52b7e8"
fi

# WITH_VISION=1: the Gemma 4 12B multimodal projector (175 MB) from the same
# pinned repo and revision as the GGUF, for llama-server --mmproj (art
# captions, scripts/art_captions.py).
if [[ "${WITH_VISION:-0}" == "1" ]]; then
  fetch google/gemma-4-12B-it-qat-q4_0-gguf 29d097773436b69ff9feafd636ab4cf873786537 "$ROOT/llm" \
    "mmproj-gemma-4-12b-it-qat-q4_0.gguf|mmproj-gemma-4-12b-it-qat-q4_0.gguf|cb018338a7538a9814d994bfe54644c71eb7ed54e31eae2f721e45fd3c260da7"
fi

# WITH_E2B=1: unsloth's Q4_K_M GGUF of google/gemma-4-E2B-it (Google publishes
# only a QAT Q4_0 of E2B; Q4_K_M is the quant the VPS CPU benchmark used);
# main as of 2026-07-17.
if [[ "${WITH_E2B:-0}" == "1" ]]; then
  fetch unsloth/gemma-4-E2B-it-GGUF 0314792d7f1f7e229411f620751375812bb9faf2 "$ROOT/llm" \
    "gemma-4-E2B-it-Q4_K_M.gguf|gemma-4-e2b-it-q4_k_m.gguf|740185b21d22ceb83a11c3aa62ad5842ef32c70f6096d756bbee85a1e4ec34b8"
fi
