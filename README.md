# pulse-proto

Protobuf schema of **Eira Pulse**, the gRPC transaction stream for Robinhood Chain (Arbitrum Nitro,
chain id 4663). This repository is the source of truth for the public schema; the copies on
[eiranodes.dev/docs/pulse](https://eiranodes.dev/docs/pulse) and in
[robinhood-feed-bench](https://github.com/qskateboard/robinhood-feed-bench) are pulled from here.

- Endpoint: `pulse.eiranodes.dev:443`, TLS, service `robin.pulse.v1.Pulse`.
- Documentation: commitment levels, filters, limits, delivery guarantees and client examples in
  Rust, Go, TypeScript and Python are at [eiranodes.dev/docs/pulse](https://eiranodes.dev/docs/pulse);
  a first connection in five minutes at [eiranodes.dev/docs/quickstart](https://eiranodes.dev/docs/quickstart).

## Using it

Vendor `pulse.proto` or add this repository as a git submodule and point your protobuf compiler at it:

```
git submodule add https://github.com/qskateboard/pulse-proto.git proto
protoc -I proto --go_out=. proto/pulse.proto            # Go
tonic_prost_build::compile_protos("proto/pulse.proto")  # Rust (build.rs)
```

`grpcurl` works against the endpoint directly: see the quickstart for the exact invocation.

## Sending transactions

`SendTransaction` (v1.2.0) takes a signed transaction and returns its hash; with `wait_for_received`
it also returns the block and position at which Eira observed it at `RECEIVED` level, usually well
before the receipt is available. A definite rejection by the sequencer comes back as
`INVALID_ARGUMENT` or `FAILED_PRECONDITION` with the sequencer's text; `UNAVAILABLE` means the
outcome is unknown and the receipt must be checked before the nonce is reused. Send never retries
on its own.

## Versioning

Tags `vMAJOR.MINOR.PATCH`. Fields are only ever added, never renumbered or removed, so any client
built from an older tag keeps working and simply ignores fields it does not know. `MINOR` grows when
fields or enum values are added, `PATCH` for comment-only changes, `MAJOR` is reserved for a new
package version (`robin.pulse.v2`). Every change is listed in [CHANGELOG.md](CHANGELOG.md).
CI checks every push with `buf breaking` against the previous tag.

## License

Apache-2.0, see [LICENSE](LICENSE).
