// Pulse subscription on the plaintext endpoint: one level, an optional address filter, reconnects.
//
//	go run .                      # every transaction at RECEIVED
//	go run . 0xADDRESS ...        # only transactions calling / emitting from these addresses
//	PULSE_LEVEL=processed go run .
package main

import (
	"context"
	"encoding/hex"
	"fmt"
	"log"
	"os"
	"strings"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	pb "github.com/qskateboard/pulse-proto/examples/go/pulsepb"
)

func main() {
	endpoint := getenv("PULSE_ENDPOINT", "pulse.eiranodes.dev:8443")
	level := pb.Commitment_RECEIVED
	switch strings.ToLower(os.Getenv("PULSE_LEVEL")) {
	case "processed":
		level = pb.Commitment_PROCESSED
	case "confirmed":
		level = pb.Commitment_CONFIRMED
	}
	var addresses [][]byte
	for _, a := range os.Args[1:] {
		b, err := hex.DecodeString(strings.TrimPrefix(a, "0x"))
		if err != nil || len(b) != 20 {
			log.Fatalf("not a 20-byte hex address: %s", a)
		}
		addresses = append(addresses, b)
	}

	conn, err := grpc.NewClient(endpoint,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithDefaultCallOptions(grpc.MaxCallRecvMsgSize(64<<20)))
	if err != nil {
		log.Fatal(err)
	}
	defer conn.Close()
	client := pb.NewPulseClient(conn)

	backoff := 500 * time.Millisecond
	for {
		got, err := subscribe(client, level, addresses)
		log.Printf("stream: %v", err)
		if got {
			backoff = 500 * time.Millisecond
		}
		log.Printf("reconnecting in %v", backoff)
		time.Sleep(backoff)
		backoff = min(backoff*2, 5*time.Second)
	}
}

// subscribe runs one subscription until it fails; got reports whether any update arrived on it.
func subscribe(client pb.PulseClient, level pb.Commitment, addresses [][]byte) (got bool, err error) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	stream, err := client.Subscribe(ctx)
	if err != nil {
		return false, err
	}
	// Named filters are OR'ed; an empty filter selects every transaction.
	name := "all"
	if len(addresses) > 0 {
		name = "watch"
	}
	err = stream.Send(&pb.SubscribeRequest{
		RequestId: 1,
		Action: &pb.SubscribeRequest_Replace{Replace: &pb.Subscription{
			Transactions: map[string]*pb.TransactionFilter{name: {AddressInclude: addresses}},
			Commitment:   level,
		}},
	})
	if err != nil {
		return false, err
	}
	// The request side stays open: later requests (a new filter set, a ping) go through stream.Send.
	for {
		update, err := stream.Recv()
		if err != nil {
			return got, err
		}
		got = true
		handle(update)
	}
}

func handle(update *pb.SubscribeUpdate) {
	switch u := update.Update.(type) {
	case *pb.SubscribeUpdate_Transaction:
		tx := u.Transaction
		if tx.Commitment == pb.Commitment_RECEIVED {
			fmt.Printf("RECEIVED  block %d #%d 0x%x to 0x%x [%s]\n",
				tx.FeedSequence, tx.TransactionIndex, tx.TransactionHash, tx.To, strings.Join(update.Filters, ","))
		} else {
			fmt.Printf("%-9s block %d #%d 0x%x logs %d success %v\n",
				tx.Commitment, tx.BlockNumber, tx.TransactionIndex, tx.TransactionHash, len(tx.Logs), tx.Success)
		}
	case *pb.SubscribeUpdate_Batch:
		// With Subscription.batch set, consecutive transactions arrive grouped; each element is a full update.
		for _, item := range u.Batch.Updates {
			handle(item)
		}
	case *pb.SubscribeUpdate_Ack:
		log.Printf("subscribed: %d filter(s), level %v", u.Ack.FilterCount, u.Ack.Commitment)
	case *pb.SubscribeUpdate_SourceStatus:
		log.Printf("RECEIVED source connected=%v epoch=%d", u.SourceStatus.Connected, u.SourceStatus.SourceEpoch)
	case *pb.SubscribeUpdate_BlockStatus:
		// PROCESSED consumers drop the events of an invalidated attempt; SEALED needs no action here.
		if u.BlockStatus.Status == pb.BlockStatus_INVALIDATED {
			log.Printf("block %d attempt %d INVALIDATED: drop its PROCESSED events", u.BlockStatus.BlockNumber, u.BlockStatus.Attempt)
		}
	}
}

func getenv(key, fallback string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return fallback
}
