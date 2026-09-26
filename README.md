# Eira Pulse — Robinhood Chain gRPC transaction stream

Protobuf schema and client examples for **[Eira Pulse](https://eiranodes.dev/robinhood-chain-transaction-feed)**, a
low-latency gRPC transaction feed for Robinhood Chain (Arbitrum Nitro, chain id 4663). Pulse delivers every
transaction the moment the sequencer orders it (RECEIVED), its executed logs before the block is sealed
(PROCESSED) and the sealed block (CONFIRMED), with address filters, over one subscription. Robinhood Chain has
no public mempool, so RECEIVED is the earliest point at which a transaction is visible
([how that works](https://eiranodes.dev/robinhood-chain-mempool)).

This repository is the source of truth for the public schema; the copies on
[eiranodes.dev/docs/pulse](https://eiranodes.dev/docs/pulse) and in
[robinhood-feed-bench](https://github.com/qskateboard/robinhood-feed-bench) are pulled from here.

- Endpoint: `pulse.eiranodes.dev:8443`, plaintext gRPC (no TLS), service `robin.pulse.v1.Pulse`. Connections
  are accepted from the source IP registered for your plan ([plans](https://eiranodes.dev/pricing), access via
  [Telegram](https://t.me/eiranodes_bot)).
- Examples: runnable subscribers in Rust, Go, Python and Node.js in [examples/](examples).
- Documentation: commitment levels, filters, limits, delivery guarantees and client examples in
  Rust, Go, TypeScript and Python are at [eiranodes.dev/docs/pulse](https://eiranodes.dev/docs/pulse);
  a first connection in five minutes at [eiranodes.dev/docs/quickstart](https://eiranodes.dev/docs/quickstart).
- Measurements: dated results at [eiranodes.dev/docs/benchmarks](https://eiranodes.dev/docs/benchmarks); run
  your own with [robinhood-feed-bench](https://github.com/qskateboard/robinhood-feed-bench).

## Using it

Vendor `pulse.proto` or add this repository as a git submodule and point your protobuf compiler at it:

```
git submodule add https://github.com/qskateboard/pulse-proto.git proto
protoc -I proto --go_out=. proto/pulse.proto            # Go
tonic_prost_build::compile_protos("proto/pulse.proto")  # Rust (build.rs)
```

`grpcurl -plaintext` works against the endpoint directly: see the quickstart for the exact invocation.

## Sending transactions

`SendTransaction` (v1.2.0) is part of the schema but is not served on the plaintext endpoint. Submit signed
transactions through [Eira Send over HTTP](https://eiranodes.dev/docs/send-http).

## Versioning

Tags `vMAJOR.MINOR.PATCH`. Fields are only ever added, never renumbered or removed, so any client
built from an older tag keeps working and simply ignores fields it does not know. `MINOR` grows when
fields or enum values are added, `PATCH` for comment-only changes, `MAJOR` is reserved for a new
package version (`robin.pulse.v2`). Every change is listed in [CHANGELOG.md](CHANGELOG.md).
CI checks every push with `buf breaking` against the previous tag.

## License

Apache-2.0, see [LICENSE](LICENSE).
