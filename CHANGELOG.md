# Changelog

## v1.2.0 — 2026-09-20

- `Pulse.SendTransaction`: submit a signed transaction through Eira and, with `wait_for_received`,
  receive its `feed_sequence` and `transaction_index` as soon as it is observed at `RECEIVED` level.
  New messages `SendTransactionRequest`, `SendTransactionResponse`, `ReceivedPosition`. Error
  semantics are documented on the messages. Available on plans that include sending.

## v1.1.0 — 2026-09-19

- `Transaction.timestamp` (18) and `Transaction.l1_block_number` (19) on `RECEIVED` events: block
  timestamp in seconds and the parent-chain block number the sequencer recorded for the block. Both
  are set on every `RECEIVED` event of the block.

## v1.0.0 — 2026-09-18

- First public schema: `Subscribe` bidirectional stream, `Commitment` levels `PROCESSED`, `CONFIRMED`,
  `RECEIVED`, transaction filters scoped to log emitters, `BlockStatus`, `SourceStatus`, `Pong`,
  `SubscriptionAck`, `raw_tx` and `value` on `RECEIVED` events.
