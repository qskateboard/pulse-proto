# Pulse client examples

Minimal subscribers for Eira Pulse in Rust, Go, Python and Node.js. Each one connects to
`pulse.eiranodes.dev:8443`, subscribes at one commitment level, optionally filters by address, prints every
transaction and reconnects with a short backoff (0.5 s, doubling up to 5 s).

The endpoint is plaintext gRPC, without TLS: every example opens an insecure channel. Connections are accepted
only from the source IP registered for your plan, see the
[quickstart](https://eiranodes.dev/docs/quickstart). For the lowest latency, run in AWS us-east-2.

| Language | Run |
|---|---|
| Rust | `cd rust && cargo run --release` |
| Go | `cd go && go run .` |
| Python | `cd python && pip install -r requirements.txt && python subscribe.py` |
| Node.js | `cd node && npm install && node subscribe.js` |

Arguments are optional `0x` addresses to filter on: at `RECEIVED` an address matches the callee (`to`), at
`PROCESSED` and `CONFIRMED` a log emitter. Without arguments every transaction is delivered.

| Environment | Meaning |
|---|---|
| `PULSE_LEVEL` | `received` (default), `processed` or `confirmed` |
| `PULSE_ENDPOINT` | another address, e.g. for a test setup |

## Notes

- **One `RECEIVED` stream per plan.** A second `RECEIVED` subscription of the same plan is refused with
  `RESOURCE_EXHAUSTED` ("one RECEIVED stream per plan"). When you switch endpoints, stop the old stream first.
- **Batching.** Python and Node.js set `Subscription.batch`: transactions that follow each other closely arrive in
  one message, which cuts the per-message cost of those runtimes. Rust and Go read one transaction per message.
- **Keep the request side open.** The stream is bidirectional; the examples keep their side open so that a new
  filter set or a ping can be sent on the same subscription.
- **No replay.** After a reconnect, delivery resumes from live data; reconcile through RPC if you need every
  event. At `PROCESSED`, a `BlockStatus` `INVALIDATED` message means the events of that attempt must be dropped.
- **Sending transactions** goes through [Eira Send over HTTP](https://eiranodes.dev/docs/send-http);
  `SendTransaction` is not served on this endpoint.

## Generated code

- Rust: generated at build time by `build.rs`, with a bundled `protoc`.
- Go: `pulsepb/` is generated from `../pulse.proto` by `go/generate.sh` (protoc, protoc-gen-go, protoc-gen-go-grpc).
- Python: the stubs are generated from `../pulse.proto` on the first run.
- Node.js: the schema is loaded at run time with `@grpc/proto-loader`.
