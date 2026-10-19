-- Trevor: bank answers in the answer cache (contract v1.1 section 3a,
-- addition 2026-10-19). The publish imports the bank's pre-generated and
-- re-answered questions into `trevor_answers` as ordinary cached rows, so
-- the next asker of the same normalized question gets the banked answer
-- from the cache.
--
-- `origin` names an imported row: 'bank' (scripts/answer_bank.py) or
-- 'reanswer' (scripts/reanswer.py). NULL is an answer a worker made, or one
-- carried over from a worker answer. The carry-over never copies an
-- imported row: every publish sends the bank again, so the bank stays the
-- one source of the imported rows and a publish without it (or with
-- `TREVOR_BANK_IMPORT=0`) leaves the new version with none.
--
-- No new index: the cache key index `trevor_answers_key` (version, server,
-- qkey, era_pos) WHERE cached already serves the bank lookup, which reads
-- one question's rows at or below an era within a version.
ALTER TABLE trevor_answers
    ADD COLUMN origin TEXT CHECK (origin IN ('bank', 'reanswer'));
