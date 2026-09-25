"""Pulse subscription on the plaintext endpoint: one level, an optional address filter, batch delivery, reconnects.

    pip install -r requirements.txt
    python subscribe.py                     # every transaction at RECEIVED
    python subscribe.py 0xADDRESS ...       # only transactions calling / emitting from these addresses
    PULSE_LEVEL=processed python subscribe.py

The stubs are generated from ../../pulse.proto on the first run.
"""
import os
import pathlib
import sys
import threading
import time

HERE = pathlib.Path(__file__).resolve().parent
PROTO = HERE.parent.parent / "pulse.proto"
if not (HERE / "pulse_pb2.py").exists():
    from grpc_tools import protoc

    if protoc.main(["protoc", f"-I{PROTO.parent}", f"--python_out={HERE}", f"--grpc_python_out={HERE}", str(PROTO)]):
        sys.exit("stub generation failed")
sys.path.insert(0, str(HERE))

import grpc  # noqa: E402
import pulse_pb2 as pb  # noqa: E402
import pulse_pb2_grpc as pbg  # noqa: E402

ENDPOINT = os.environ.get("PULSE_ENDPOINT", "pulse.eiranodes.dev:8443")
LEVEL = {"processed": pb.PROCESSED, "confirmed": pb.CONFIRMED}.get(os.environ.get("PULSE_LEVEL", "").lower(), pb.RECEIVED)
ADDRESSES = [bytes.fromhex(a.removeprefix("0x")) for a in sys.argv[1:]]
assert all(len(a) == 20 for a in ADDRESSES), "addresses must be 20 bytes"


def requests(stop: threading.Event):
    sub = pb.Subscription(commitment=LEVEL, batch=True)  # batch: fewer messages, much less Python overhead
    # Named filters are OR'ed; an empty filter selects every transaction.
    sub.transactions["watch" if ADDRESSES else "all"].address_include.extend(ADDRESSES)
    yield pb.SubscribeRequest(request_id=1, replace=sub)
    stop.wait()  # keep the request side open for the life of the subscription


def handle(update) -> None:
    kind = update.WhichOneof("update")
    if kind == "transaction":
        tx = update.transaction
        if tx.commitment == pb.RECEIVED:
            print(f"RECEIVED  block {tx.feed_sequence} #{tx.transaction_index} 0x{tx.transaction_hash.hex()} "
                  f"to 0x{tx.to.hex()} [{','.join(update.filters)}]")
        else:
            print(f"{pb.Commitment.Name(tx.commitment):<9} block {tx.block_number} #{tx.transaction_index} "
                  f"0x{tx.transaction_hash.hex()} logs {len(tx.logs)} success {tx.success}")
    elif kind == "batch":  # consecutive transactions grouped; each element is a full update
        for item in update.batch.updates:
            handle(item)
    elif kind == "ack":
        print(f"subscribed: {update.ack.filter_count} filter(s), level {pb.Commitment.Name(update.ack.commitment)}",
              file=sys.stderr)
    elif kind == "source_status":
        s = update.source_status
        print(f"RECEIVED source connected={s.connected} epoch={s.source_epoch}", file=sys.stderr)
    elif kind == "block_status" and update.block_status.status == pb.BlockStatus.INVALIDATED:
        # PROCESSED consumers drop the events of an invalidated attempt; SEALED needs no action here.
        b = update.block_status
        print(f"block {b.block_number} attempt {b.attempt} INVALIDATED: drop its PROCESSED events", file=sys.stderr)


def main() -> None:
    channel = grpc.insecure_channel(ENDPOINT, options=[("grpc.max_receive_message_length", 64 << 20)])
    stub = pbg.PulseStub(channel)
    backoff = 0.5
    while True:
        stop = threading.Event()
        got = False
        try:
            for update in stub.Subscribe(requests(stop)):
                got = True
                handle(update)
            print("stream ended by the server", file=sys.stderr)
        except grpc.RpcError as e:
            print(f"stream: {e.code().name} {e.details()}", file=sys.stderr)
        finally:
            stop.set()
        if got:
            backoff = 0.5
        print(f"reconnecting in {backoff:.1f} s", file=sys.stderr)
        time.sleep(backoff)
        backoff = min(backoff * 2, 5.0)


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        pass
