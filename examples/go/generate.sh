#!/bin/sh
# Regenerate pulsepb/ from ../../pulse.proto. Needs protoc, protoc-gen-go and protoc-gen-go-grpc:
#   go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
#   go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest
set -eu
cd "$(dirname "$0")"
PKG=github.com/qskateboard/pulse-proto/examples/go/pulsepb
mkdir -p pulsepb
protoc -I ../.. \
  --go_out=pulsepb --go_opt=paths=source_relative --go_opt=Mpulse.proto=$PKG \
  --go-grpc_out=pulsepb --go-grpc_opt=paths=source_relative --go-grpc_opt=Mpulse.proto=$PKG \
  ../../pulse.proto
