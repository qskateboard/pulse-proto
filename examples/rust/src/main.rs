//! Pulse subscription on the plaintext endpoint: one level, an optional address filter, reconnects.
//!
//!     cargo run --release                      # every transaction at RECEIVED
//!     cargo run --release -- 0xADDRESS ...     # only transactions calling / emitting from these addresses
//!     PULSE_LEVEL=processed cargo run --release
use std::time::Duration;

use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Channel;

pub mod pb {
    tonic::include_proto!("robin.pulse.v1");
}
use pb::pulse_client::PulseClient;
use pb::subscribe_update::Update;
use pb::{subscribe_request, Commitment, SubscribeRequest, SubscribeUpdate, Subscription, TransactionFilter};

const ENDPOINT: &str = "http://pulse.eiranodes.dev:8443";

#[tokio::main]
async fn main() {
    let endpoint = std::env::var("PULSE_ENDPOINT").unwrap_or_else(|_| ENDPOINT.to_string());
    let level = match std::env::var("PULSE_LEVEL").unwrap_or_default().to_lowercase().as_str() {
        "processed" => Commitment::Processed,
        "confirmed" => Commitment::Confirmed,
        _ => Commitment::Received,
    };
    let addresses: Vec<Vec<u8>> = std::env::args().skip(1).map(|a| address(&a)).collect();
    let mut backoff = Duration::from_millis(500);
    loop {
        let mut got = false;
        match subscribe(&endpoint, level, &addresses, &mut got).await {
            Ok(()) => eprintln!("stream ended by the server"),
            Err(e) => eprintln!("stream: {e}"),
        }
        if got {
            backoff = Duration::from_millis(500);
        }
        eprintln!("reconnecting in {backoff:?}");
        tokio::time::sleep(backoff).await;
        backoff = (backoff * 2).min(Duration::from_secs(5));
    }
}

async fn subscribe(endpoint: &str, level: Commitment, addresses: &[Vec<u8>], got: &mut bool) -> Result<(), Box<dyn std::error::Error>> {
    let channel = Channel::from_shared(endpoint.to_string())?.tcp_nodelay(true).connect().await?;
    let mut client = PulseClient::new(channel).max_decoding_message_size(64 << 20);

    // Named filters are OR'ed; an empty filter selects every transaction.
    let name = if addresses.is_empty() { "all" } else { "watch" };
    let filter = TransactionFilter { address_include: addresses.to_vec(), ..Default::default() };
    let request = SubscribeRequest {
        request_id: 1,
        action: Some(subscribe_request::Action::Replace(Subscription {
            transactions: [(name.to_string(), filter)].into(),
            commitment: level as i32,
            ..Default::default()
        })),
    };
    // The request side stays open for the life of the subscription: later requests (a new filter set,
    // a ping) go through `requests`.
    let (requests, rx) = mpsc::channel(4);
    requests.send(request).await?;
    let mut updates = client.subscribe(ReceiverStream::new(rx)).await?.into_inner();
    while let Some(update) = updates.message().await? {
        *got = true;
        handle(update);
    }
    Ok(())
}

fn handle(update: SubscribeUpdate) {
    match update.update {
        Some(Update::Transaction(tx)) => {
            if tx.commitment == Commitment::Received as i32 {
                println!(
                    "RECEIVED  block {} #{} 0x{} to 0x{} [{}]",
                    tx.feed_sequence, tx.transaction_index, hex(&tx.transaction_hash), hex(&tx.to), update.filters.join(",")
                );
            } else {
                println!(
                    "{:<9} block {} #{} 0x{} logs {} success {}",
                    level_name(tx.commitment),
                    tx.block_number, tx.transaction_index, hex(&tx.transaction_hash), tx.logs.len(), tx.success
                );
            }
        }
        // With Subscription.batch set, consecutive transactions arrive grouped; each element is a full update.
        Some(Update::Batch(batch)) => batch.updates.into_iter().for_each(handle),
        Some(Update::Ack(ack)) => eprintln!("subscribed: {} filter(s), level {}", ack.filter_count, level_name(ack.commitment)),
        Some(Update::SourceStatus(s)) => eprintln!("RECEIVED source connected={} epoch={}", s.connected, s.source_epoch),
        // PROCESSED consumers drop the events of an invalidated attempt; SEALED needs no action here.
        Some(Update::BlockStatus(b)) if b.status == pb::block_status::Status::Invalidated as i32 => {
            eprintln!("block {} attempt {} INVALIDATED: drop its PROCESSED events", b.block_number, b.attempt)
        }
        Some(Update::BlockStatus(_)) | Some(Update::Pong(_)) | None => {}
    }
}

fn level_name(level: i32) -> &'static str {
    Commitment::try_from(level).map(|c| c.as_str_name()).unwrap_or("UNKNOWN")
}

fn address(s: &str) -> Vec<u8> {
    let s = s.trim_start_matches("0x");
    let bytes: Vec<u8> = (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(s.get(i..i + 2).unwrap_or("zz"), 16).expect("address must be hex"))
        .collect();
    assert_eq!(bytes.len(), 20, "address must be 20 bytes");
    bytes
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
