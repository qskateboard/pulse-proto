# Changelog

## v1.3.0 — 2026-09-24

- `Subscription.batch` (3) and `SubscribeUpdate.batch` (9, new message `UpdateBatch`): on a `RECEIVED`
  subscription, transactions arrive grouped, each element being the complete update that would otherwise be
  sent on its own. The first transaction is sent at once, the ones that closely follow it are grouped; the
  set and order of transactions are unchanged. Meant for clients with a high per-message cost such as
  Python `grpcio`. Off by default: clients that do not set it receive exactly what they received before.
  Code that builds `Subscription` with an exhaustive struct literal (Rust `prost`) needs
  `..Default::default()` or `batch: false` after regenerating.

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
