
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

The dummysigner handles revault descriptors

```
cargo run -- --conf  <config_path>
```

You can find an example of the configuration file
[here](examples/examples_cfg.toml).

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
  "spend_tx": "<base64 encoded psbt>" 
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
  "unvault_tx": "<base64 encoded psbt>" 
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
  "cancel_tx": "<base64 encoded psbt>",
  "emergency_tx": "<base64 encoded psbt>",
  "emergency_unvault_tx": "<base64 encoded psbt>"
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

### Secure deposits in batch

This method requires the descriptors and the emergency address.

#### request:

```json
{
  "deposits": [
    {
      "outpoint": "<txid>:<vout>",
      "amount": "<amount in satoshis>",
      "derivation_index": "<derivation index>"
    },
    ...
  ]
}
```

#### response:

```json
[
  {
    "cancel_tx": "<base64 encoded psbt>",
    "emergency_tx": "<base64 encoded psbt>",
    "emergency_unvault_tx": "<base64 encoded psbt>"
  },
  ...
]
```

### Delegate vaults in batch 

This method requires the descriptors and the emergency address.

#### request:

```json
{
  "vaults": [
    {
      "outpoint": "<txid>:<vout>",
      "amount": "<amount in satoshis>",
      "derivation_index": "<derivation index>"
    },
    ...
  ]
}
```

#### response:

```json
[
  {
    "unvault_tx": "<base64 encoded psbt>"
  },
  ...
]
```

## Example

```
cargo run -- --conf examples/examples_cfg.toml
```

then run the client:

```
cargo run --example stakeholder_batch
```
