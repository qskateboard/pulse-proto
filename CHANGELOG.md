# Changelog

## v1.1.0 — 2026-09-19

- `Transaction.timestamp` (18) and `Transaction.l1_block_number` (19) on `RECEIVED` events: block
  timestamp in seconds and the parent-chain block number the sequencer recorded for the block. Both
  are set on every `RECEIVED` event of the block.

## v1.0.0 — 2026-09-18

- First public schema: `Subscribe` bidirectional stream, `Commitment` levels `PROCESSED`, `CONFIRMED`,
  `RECEIVED`, transaction filters scoped to log emitters, `BlockStatus`, `SourceStatus`, `Pong`,
  `SubscriptionAck`, `raw_tx` and `value` on `RECEIVED` events.
