// Pulse subscription on the plaintext endpoint: one level, an optional address filter, batch delivery, reconnects.
//
//   npm install
//   node subscribe.js                     # every transaction at RECEIVED
//   node subscribe.js 0xADDRESS ...       # only transactions calling / emitting from these addresses
//   PULSE_LEVEL=processed node subscribe.js
const path = require("path");
const grpc = require("@grpc/grpc-js");
const loader = require("@grpc/proto-loader");

const definition = loader.loadSync(path.join(__dirname, "..", "..", "pulse.proto"), {
  keepCase: true,
  longs: String,
  enums: String,
  defaults: true,
  oneofs: true,
});
const { Pulse } = grpc.loadPackageDefinition(definition).robin.pulse.v1;

const ENDPOINT = process.env.PULSE_ENDPOINT || "pulse.eiranodes.dev:8443";
const LEVEL = { processed: "PROCESSED", confirmed: "CONFIRMED" }[(process.env.PULSE_LEVEL || "").toLowerCase()] || "RECEIVED";
const ADDRESSES = process.argv.slice(2).map((a) => {
  const b = Buffer.from(a.replace(/^0x/, ""), "hex");
  if (b.length !== 20) throw new Error(`not a 20-byte hex address: ${a}`);
  return b;
});

const client = new Pulse(ENDPOINT, grpc.credentials.createInsecure(), {
  "grpc.max_receive_message_length": 64 << 20,
});

let backoff = 500;

function subscribe() {
  const call = client.Subscribe();
  let got = false;
  let done = false;
  const retry = (why) => {
    if (done) return;
    done = true;
    console.error(`stream: ${why}`);
    if (got) backoff = 500;
    console.error(`reconnecting in ${backoff} ms`);
    setTimeout(subscribe, backoff);
    backoff = Math.min(backoff * 2, 5000);
  };
  call.on("data", (update) => {
    got = true;
    handle(update);
  });
  call.on("error", (e) => retry(`${grpc.status[e.code]} ${e.details}`));
  call.on("end", () => retry("ended by the server"));
  // Named filters are OR'ed; an empty filter selects every transaction. batch: fewer messages.
  // The call stays open for writing: later requests (a new filter set, a ping) go through call.write.
  call.write({
    request_id: 1,
    replace: {
      transactions: { [ADDRESSES.length ? "watch" : "all"]: { address_include: ADDRESSES } },
      commitment: LEVEL,
      batch: true,
    },
  });
}

function handle(update) {
  switch (update.update) {
    case "transaction": {
      const tx = update.transaction;
      if (tx.commitment === "RECEIVED") {
        console.log(`RECEIVED  block ${tx.feed_sequence} #${tx.transaction_index} 0x${tx.transaction_hash.toString("hex")} to 0x${tx.to.toString("hex")} [${update.filters.join(",")}]`);
      } else {
        console.log(`${tx.commitment.padEnd(9)} block ${tx.block_number} #${tx.transaction_index} 0x${tx.transaction_hash.toString("hex")} logs ${tx.logs.length} success ${tx.success}`);
      }
      break;
    }
    case "batch": // consecutive transactions grouped; each element is a full update
      for (const item of update.batch.updates) handle(item);
      break;
    case "ack":
      console.error(`subscribed: ${update.ack.filter_count} filter(s), level ${update.ack.commitment}`);
      break;
    case "source_status":
      console.error(`RECEIVED source connected=${update.source_status.connected} epoch=${update.source_status.source_epoch}`);
      break;
    case "block_status":
      // PROCESSED consumers drop the events of an invalidated attempt; SEALED needs no action here.
      if (update.block_status.status === "INVALIDATED") {
        console.error(`block ${update.block_status.block_number} attempt ${update.block_status.attempt} INVALIDATED: drop its PROCESSED events`);
      }
      break;
  }
}

subscribe();
