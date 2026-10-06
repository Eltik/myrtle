#!/usr/bin/env python3
"""Fine-tune gte-modernbert-base on Trevor's (question, passage) pairs and export it the way Trevor loads it.

Run with the training venv: models/ft-venv/bin/python scripts/embed_finetune.py STAGE
  export-base  the original model (pinned revision, models/gte-modernbert-base-hf) exported and quantized with the
               same pipeline as the fine-tuned one -> models/gte-requant. The shipped model_int8.onnx came from a
               different quantization run, so the fair comparison is fine-tuned against this, not against it.
  train        pairs from scripts/embed_pairs.py plus the answer bank's 900 questions (both outside gold set v2's
               stories), cached multiple-negatives ranking loss (in-batch negatives at batch 64), 1 epoch
               -> models/gte-trevor-hf
  export       -> models/gte-trevor
Export: torch.onnx.export at opset 14, inputs input_ids and attention_mask, output last_hidden_state (the shipped
file's signature), then onnxruntime dynamic quantization (DynamicQuantizeLinear and MatMulInteger, as shipped).
The tokenizer files are copied from models/gte-modernbert-base: chunk token counts are pinned to that tokenizer's
sha, and fine-tuning does not change it.
"""
import os, random, shutil, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))  # common.py and incr.py sit beside the scripts
from common import ROOT, read_jsonl

M = os.path.join(ROOT, 'models')
BASE_HF = os.path.join(M, 'gte-modernbert-base-hf')
FT_HF = os.path.join(M, 'gte-trevor-hf')
MAX_LEN = 512


def export(src, out):
    import torch
    from transformers import AutoModel
    from onnxruntime.quantization import QuantType, quantize_dynamic
    os.makedirs(out, exist_ok=True)
    model = AutoModel.from_pretrained(src, attn_implementation='eager', dtype=torch.float32).eval().float()

    class Wrap(torch.nn.Module):
        def __init__(self, m):
            super().__init__()
            self.m = m

        def forward(self, input_ids, attention_mask):
            return self.m(input_ids=input_ids, attention_mask=attention_mask).last_hidden_state

    ids = torch.ones(1, 16, dtype=torch.long)
    fp32 = os.path.join(out, 'model.onnx')
    torch.onnx.export(Wrap(model), (ids, torch.ones_like(ids)), fp32, input_names=['input_ids', 'attention_mask'],
                      output_names=['last_hidden_state'], opset_version=14, dynamo=False,
                      dynamic_axes={'input_ids': {0: 'batch_size', 1: 'sequence_length'},
                                    'attention_mask': {0: 'batch_size', 1: 'sequence_length'},
                                    'last_hidden_state': {0: 'batch_size', 1: 'sequence_length'}})
    # MatMul only: the shipped file quantizes 88 MatMuls and leaves the embedding Gather in float. Signed per-channel
    # weights: mean cosine to the fp32 export 0.9988 on 6 texts, against 0.9967 for the shipped INT8 file and 0.949
    # for the unsigned per-tensor default.
    quantize_dynamic(fp32, os.path.join(out, 'model_int8.onnx'), weight_type=QuantType.QInt8, per_channel=True,
                     op_types_to_quantize=['MatMul'])
    for f in ('tokenizer.json', 'tokenizer_config.json', 'special_tokens_map.json', 'config.json'):
        shutil.copy(os.path.join(M, 'gte-modernbert-base', f), os.path.join(out, f))
    print(f'exported {src} -> {out}')


def train():
    from datasets import Dataset
    from sentence_transformers import SentenceTransformer, SentenceTransformerTrainer, SentenceTransformerTrainingArguments
    from sentence_transformers.losses import CachedMultipleNegativesRankingLoss
    from sentence_transformers.training_args import BatchSamplers
    chunks = {r['chunkId']: r['text'] for r in read_jsonl(os.path.join(ROOT, 'artifacts', 'p3a', 'chunks.jsonl'))}
    gold = {an['story_id'] for g in read_jsonl(os.path.join(ROOT, 'eval', 'goldset_v2.jsonl')) for an in g.get('anchors', [])}
    rows = read_jsonl(os.path.join(ROOT, 'artifacts', 'embedft', 'pairs.jsonl')) + \
        read_jsonl(os.path.join(ROOT, 'artifacts', 'bank', 'questions.jsonl'))
    pairs = [(r['question'], chunks[r['chunkId']]) for r in rows
             if r['chunkId'] in chunks and r['storyId'] not in gold and len(r.get('question', '').split()) >= 2]
    random.Random(3).shuffle(pairs)
    cut = max(1, len(pairs) // 20)
    tr = Dataset.from_dict({'anchor': [q for q, _ in pairs[cut:]], 'positive': [p for _, p in pairs[cut:]]})
    ev = Dataset.from_dict({'anchor': [q for q, _ in pairs[:cut]], 'positive': [p for _, p in pairs[:cut]]})
    print(f'pairs: {len(pairs)} (train {len(tr)}, eval {len(ev)}); gold stories excluded {len(gold)}', flush=True)
    # float32: the checkpoint is stored in float16 (config torch_dtype), and the first run trained in it on MPS with no
    # loss scaling; every one of the 134 weight tensors came out non-finite (eval loss NaN from step 20).
    import torch
    model = SentenceTransformer(BASE_HF, device='mps', model_kwargs={'attn_implementation': 'eager', 'dtype': torch.float32})
    model.max_seq_length = MAX_LEN
    loss = CachedMultipleNegativesRankingLoss(model, mini_batch_size=8)
    args = SentenceTransformerTrainingArguments(
        output_dir=os.path.join(M, 'ft-checkpoints'), num_train_epochs=1, per_device_train_batch_size=64,
        per_device_eval_batch_size=64, learning_rate=2e-5, warmup_ratio=0.1, batch_sampler=BatchSamplers.NO_DUPLICATES,
        eval_strategy='steps', eval_steps=20, logging_steps=10, save_strategy='no', report_to=[], seed=3)
    SentenceTransformerTrainer(model=model, args=args, train_dataset=tr, eval_dataset=ev, loss=loss).train()
    model.save(FT_HF)
    shutil.rmtree(os.path.join(M, 'ft-checkpoints'), ignore_errors=True)
    print(f'saved {FT_HF}')


if __name__ == '__main__':
    stage = sys.argv[1]
    if stage == 'export-base':
        export(BASE_HF, os.path.join(M, 'gte-requant'))
    elif stage == 'train':
        train()
    elif stage == 'export':
        export(FT_HF, os.path.join(M, 'gte-trevor'))
