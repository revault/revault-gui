
# `dummysigner`

A simple signer to simulate Hardware wallet usage with
revault-gui.

Do not use with real funds.

## Usage

The dummysigner can sign with multiple extended private key following
the [bip32](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki) format.

```
cargo run -- <xpriv> <xpriv> ... 
```

## Communication

### Transport

`dummysigner` use a tcp server listening to `0.0.0.0:8080` in order to receive and respond to signature
requests.

Messages are json objects framed by the [tokio_util length delimited
codec](https://docs.rs/tokio-util/0.6.7/tokio_util/codec/length_delimited/index.html). 

### Request refused

If the signature request was refused the response looks like:

```json
{"request_status": "refused"}
```

### Sign spend transaction

#### request:

```json
{
  "derivation_path": ["<string ex: m/1>"], // paths ordered by respective psbt input index
  "target": {
    "spend_tx": "<base64 encoded psbt>" 
  }
}
```

#### response:

```json
{
  "spend_tx": "<base64 encoded psbt>" 
}
```

### Sign unvault transaction

#### request:

```json
{
  "derivation_path": "<string ex: m/1>",
  "target": {
    "unvault_tx": "<base64 encoded psbt>" 
  }
}
```

#### response:

```json
{
  "unvault_tx": "<base64 encoded psbt>" 
}
```

### Sign revocation transactions

#### request:

```json
{
  "derivation_path": "<string ex: m/1>",
  "target": {
    "cancel_tx": "<base64 encoded psbt>",
    "emergency_tx": "<base64 encoded psbt>",
    "emergency_unvault_tx": "<base64 encoded psbt>"
  }
}
```

#### response:

```json
{
  "cancel_tx": "<base64 encoded psbt>",
  "emergency_tx": "<base64 encoded psbt>",
  "emergency_unvault_tx": "<base64 encoded psbt>"
}
```

## Example

```
cargo run -- xprv9zFeRZgUZaUZBEUq1vPFLpUavHPK5YZ6N2qeqCYe7GLxGVY9SRHuN5Uwd5YN56tMUKe2qPhmvP8fC1GBEAFRAwbJQi86swWvvGM5tXBpJt6
```

then run the client:

```
cargo run --example stakeholder_client
```
