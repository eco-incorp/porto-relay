# Porto Relay Docker Setup Guide

This guide walks you through deploying and running the Porto Relay with docker-compose, including all necessary prerequisites and configuration steps.

## Table of Contents

-   [Prerequisites](#prerequisites)
-   [Step 1: Deploy SimpleSettler Contracts](#step-1-deploy-simplesettler-contracts)
-   [Step 2: Mint Lit PKP](#step-2-mint-lit-pkp)
-   [Step 3: Add PKP to Crowd Liquidity](#step-3-add-pkp-to-crowd-liquidity)
-   [Step 4: Configure Environment Variables](#step-4-configure-environment-variables)
-   [Step 5: Configure relay.yaml](#step-5-configure-relayyaml)
-   [Step 6: Run the Services](#step-6-run-the-services)
-   [Verification](#verification)
-   [Architecture Overview](#architecture-overview)
-   [Troubleshooting](#troubleshooting)

## Prerequisites

Before starting, ensure you have:

-   **Docker** (v20.10+) and **Docker Compose** (v2.0+) installed
-   **Foundry** (for contract deployment): Install from [getfoundry.sh](https://getfoundry.sh/)
-   **RPC endpoints** for your target chains (Base, Optimism, etc.)
-   **CoinGecko API key**: Get one at [coingecko.com/en/api/pricing](https://www.coingecko.com/en/api/pricing)
-   **Private keys** for:
    -   Relay transaction signing (as mnemonic)
    -   Funder account
    -   Settler account (for cross-chain operations)
-   **Test funds** on your target chains for gas

## Step 1: Deploy SimpleSettler Contracts

The SimpleSettler contract handles cross-chain settlement. You need to deploy it on each chain you want to support.

### Get the Contract

The SimpleSettler contract is available here:

```
https://github.com/ithacaxyz/account/blob/b3eb59f091a4491f7fa10448e2f82727fb8fd574/src/SimpleSettler.sol
```

### Deploy to Each Chain

For **Base Mainnet**:

```bash
cd path/to/account-repo

# Set your private key and RPC URL
export PRIVATE_KEY=your_private_key_here
export BASE_RPC_URL=https://base-mainnet.g.alchemy.com/v2/YOUR_KEY

# Deploy SimpleSettler
forge create src/SimpleSettler.sol:SimpleSettler \
    --rpc-url $BASE_RPC_URL \
    --private-key $PRIVATE_KEY

# Save the deployed address - you'll need it for relay.yaml
```

For **Optimism Mainnet**:

```bash
export OP_RPC_URL=https://opt-mainnet.g.alchemy.com/v2/YOUR_KEY

forge create src/SimpleSettler.sol:SimpleSettler \
    --rpc-url $OP_RPC_URL \
    --private-key $PRIVATE_KEY

# Save this address too
```

**Note:** Make sure to save both deployed addresses - you'll need them in Step 5.

## Step 2: Mint Lit PKP

Lit Protocol Programmable Key Pairs (PKPs) are used for decentralized signing in the Porto Relay.

### Mint on Datil-Test Network

1. Visit the Lit Protocol documentation or use the Lit SDK to mint a PKP on the `datil-test` network

2. You'll need to:

    - Create a PKP wallet owner account
    - Mint a new PKP
    - Obtain the PKP public key (compressed format, 130 hex characters)

3. Save the following:
    - **PKP Public Key** (e.g., `0477a05819c5ebbc5afd9fb799ff0ffb4fac069ad82bc267abe494f2524cd84e4c88f52f8829b7d037495be13164074158773af918288da14a22ed0d2d83ca1e1e`)
    - **PKP Wallet Owner Private Key**

Example using Lit SDK:

```typescript
import { LitNodeClient } from "@lit-protocol/lit-node-client";

// Initialize Lit client
const litNodeClient = new LitNodeClient({
    litNetwork: "datil-test",
});
await litNodeClient.connect();

// Mint PKP (implementation depends on your setup)
// Save the resulting PKP public key and owner private key
```

## Step 3: Add PKP to Crowd Liquidity

The PKP must be registered as a SIGNER in the crowd liquidity system.

1. Navigate to your crowd liquidity contract/admin interface
2. Add the PKP public key as an authorized SIGNER
3. Verify the PKP is properly registered and can sign transactions

This step ensures the PKP can participate in liquidity operations across chains.

## Step 4: Configure Environment Variables

Create a `.env` file in the porto-relay directory based on `.env.example`:

```bash
cd porto-relay
cp .env.example .env
```

Edit `.env` with your actual values:

```bash
# ============================================================================
# Relay Service Configuration
# ============================================================================

# CoinGecko API Key (required for price feeds)
GECKO_API=your_coingecko_api_key_here

# Relay Signer Mnemonic (12 or 24 word mnemonic phrase)
# Used for generating relay transaction signers
RELAY_MNEMONIC="word1 word2 word3 ... word12"

# Funder Signer Private Key (with 0x prefix)
# Private key for the funder account that provides liquidity
RELAY_FUNDER_SIGNER_KEY=0xYOUR_FUNDER_PRIVATE_KEY

# Settler Signer Private Key (with 0x prefix)
# Private key for cross-chain settlement operations
RELAY_SETTLER_SIGNER_KEY=0xYOUR_SETTLER_PRIVATE_KEY

# Rust Logging Level
# Options: trace, debug, info, warn, error
RUST_LOG=info

# ============================================================================
# Lit Actions Server Configuration
# ============================================================================

# PKP Public Key (from Step 2)
# This is the uncompressed public key (130 hex characters)
PKP_PUBLIC_KEY=0477a05819c5ebbc5afd9fb799ff0ffb4fac069ad82bc267abe494f2524cd84e4c88f52f8829b7d037495be13164074158773af918288da14a22ed0d2d83ca1e1e

# PKP Wallet Owner Private Key (from Step 2)
PKP_WALLET_OWNER_PRIVATE_KEY=0xYOUR_PKP_OWNER_PRIVATE_KEY
```

### Important Notes:

-   **Never commit** the `.env` file to version control
-   **RELAY_MNEMONIC**: Must be a valid BIP39 mnemonic phrase
-   **All private keys**: Should include the `0x` prefix
-   **GECKO_API**: Required for accurate price conversions between assets

## Step 5: Configure relay.yaml

The `relay.yaml` file contains the main configuration for the Porto Relay. You need to update several sections with your deployed contracts and keys.

### Key Sections to Update

#### 1. Contract Addresses

Update these with your deployed Porto account contracts:

```yaml
# Contract Addresses (Porto Mainnet Deployment)
orchestrator: "0xB447BA5a2Fb841406cdac4585Fdc28027d7Ae503"
delegation_proxy: "0x5874F358359ee96d2b3520409018f1a6F59A2CDC"
simulator: "0xcb80788813C39d90C48C2733B43b3E47e23A2D3F"
funder: "0x496fD51413B4dd4A05eeFa5E43fF7B32268c886D"
escrow: "0x55626138525a47AF075322aAfE1df8F68993b11D"
```

#### 2. Lit Actions Configuration

The endpoint should be set to the docker service name:

```yaml
lit_actions:
    endpoint: "http://lit-server:3050/execute" # Docker service name
    ipfsCid: "QmWPhfxNQLzEuu219NSYrpiB9VUkrAS9oxFS8rCZ2u4o5W"
```

**Note:** The endpoint uses `lit-server` (not `localhost`) because containers communicate via Docker networking.

#### 3. Chain Configuration with Settler Addresses

For **Base Mainnet** (chain ID 8453):

```yaml
chains:
    8453:
        endpoint: "wss://base-mainnet.g.alchemy.com/v2/YOUR_ALCHEMY_KEY"
        settler_address: "0xYOUR_BASE_SETTLER_ADDRESS" # From Step 1

        signers:
            num_signers: 1

        fees:
            signer_balance_config:
                type: balance
                value: 0

        assets:
            ethereum:
                address: "0x0000000000000000000000000000000000000000"
                decimals: 18
                fee_token: true
                interop: true
            usd-coin:
                address: "0x8C6be0b292A6DbDF81b6226E485702E6a85443e1"
                decimals: 6
                fee_token: true
                interop: true
```

For **Optimism Mainnet** (chain ID 10):

```yaml
10:
    endpoint: "wss://opt-mainnet.g.alchemy.com/v2/YOUR_ALCHEMY_KEY"
    settler_address: "0xYOUR_OP_SETTLER_ADDRESS" # From Step 1

    signers:
        num_signers: 1

    fees:
        signer_balance_config:
            type: balance
            value: 0

    assets:
        ethereum:
            address: "0x0000000000000000000000000000000000000000"
            decimals: 18
            fee_token: true
            interop: true
        usd-coin:
            address: "0x81D3B5fbBF9F076D0e353Ef8dEFd100e043CdB3e"
            decimals: 6
            fee_token: true
            interop: true
```

#### 4. Interop Configuration

Update the settler private key:

```yaml
interop:
    refund_check_interval: 1800
    escrow_refund_threshold: 7200
    funder_fee_bps: 50
    settler:
        wait_verification_timeout: 7200
        simple:
            private_key: "0xYOUR_SETTLER_PRIVATE_KEY" # Same as RELAY_SETTLER_SIGNER_KEY
```

**Important:** The `private_key` here should match your `RELAY_SETTLER_SIGNER_KEY` from the `.env` file.

### Complete Example relay.yaml

Here's a minimal complete example with placeholders:

```yaml
# Server Configuration
server:
    address: "0.0.0.0"
    port: 9119
    metrics_port: 9000
    max_connections: 100

# Contract Addresses
orchestrator: "0xYOUR_ORCHESTRATOR_ADDRESS"
delegation_proxy: "0xYOUR_DELEGATION_PROXY_ADDRESS"
simulator: "0xYOUR_SIMULATOR_ADDRESS"
funder: "0xYOUR_FUNDER_ADDRESS"
escrow: "0xYOUR_ESCROW_ADDRESS"
fee_recipient: "0xYOUR_FEE_RECIPIENT_ADDRESS"

# Lit Actions Configuration
lit_actions:
    endpoint: "http://lit-server:3050/execute"
    ipfsCid: "QmWPhfxNQLzEuu219NSYrpiB9VUkrAS9oxFS8rCZ2u4o5W"

# Chain Configuration
chains:
    8453: # Base
        endpoint: "wss://base-mainnet.g.alchemy.com/v2/YOUR_KEY"
        settler_address: "0xYOUR_BASE_SETTLER"
        signers:
            num_signers: 1
        fees:
            signer_balance_config:
                type: balance
                value: 0
        assets:
            ethereum:
                address: "0x0000000000000000000000000000000000000000"
                decimals: 18
                fee_token: true
                interop: true
            usd-coin:
                address: "0x8C6be0b292A6DbDF81b6226E485702E6a85443e1"
                decimals: 6
                fee_token: true
                interop: true

    10: # Optimism
        endpoint: "wss://opt-mainnet.g.alchemy.com/v2/YOUR_KEY"
        settler_address: "0xYOUR_OP_SETTLER"
        signers:
            num_signers: 1
        fees:
            signer_balance_config:
                type: balance
                value: 0
        assets:
            ethereum:
                address: "0x0000000000000000000000000000000000000000"
                decimals: 18
                fee_token: true
                interop: true
            usd-coin:
                address: "0x81D3B5fbBF9F076D0e353Ef8dEFd100e043CdB3e"
                decimals: 6
                fee_token: true
                interop: true

# Quote Configuration
quote:
    ttl: 5
    rateTtl: 300
    gas:
        intentBuffer: 50000
        txBuffer: 50000

# Transaction Service Configuration
transactions:
    max_pending_transactions: 100
    max_transactions_per_signer: 100
    max_queued_per_eoa: 1
    balance_check_interval: 5
    nonce_check_interval: 60
    transaction_timeout: 120
    public_node_endpoints: {}

# Price Feed Configuration
pricefeed:
    coingecko:
        remapping:
            usd-coin: "usd-coin"
            ethereum: "ethereum"

# Interop Configuration
interop:
    refund_check_interval: 1800
    escrow_refund_threshold: 7200
    funder_fee_bps: 50
    settler:
        wait_verification_timeout: 7200
        simple:
            private_key: "0xYOUR_SETTLER_PRIVATE_KEY"
```

## Step 6: Run the Services

Now that everything is configured, you can start the Porto Relay.

### Build and Start

```bash
# Navigate to porto-relay directory
cd porto-relay

# Build and start both services
docker-compose up --build
```

This will:

1. Build the relay service Docker image (takes 5-10 minutes first time)
2. Build the lit-server service Docker image
3. Start both services with health checks

### Run in Background

To run the services in detached mode:

```bash
docker-compose up --build -d
```

### View Logs

To view logs from both services:

```bash
# All services
docker-compose logs -f

# Just the relay
docker-compose logs -f relay

# Just the lit-server
docker-compose logs -f lit-server
```

### Stop Services

```bash
# Stop services
docker-compose down

# Stop and remove volumes
docker-compose down -v
```

## Verification

After starting the services, verify they're running correctly.

### Check Service Health

**Relay Health Check:**

```bash
curl http://localhost:9119/health
```

Expected response: HTTP 200 OK

**Lit Server Health Check:**

```bash
curl http://localhost:3050/health
```

Expected response: HTTP 200 OK

### Check Docker Container Status

```bash
docker ps
```

You should see two containers:

-   `porto-relay` (status: healthy)
-   `porto-lit-server` (status: healthy)

### View Metrics

The relay exposes Prometheus metrics:

```bash
curl http://localhost:9000/metrics
```

### Test RPC Endpoint

Test the relay RPC interface:

```bash
curl -X POST http://localhost:9119 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "relay_getInfo",
    "params": [],
    "id": 1
  }'
```

## Architecture Overview

### Services

#### 1. Porto Relay (`relay`)

The main relay service that:

-   Accepts cross-chain transaction requests
-   Sponsors transactions with fee abstraction
-   Manages transaction signing and submission
-   Handles cross-chain settlement via SimpleSettler
-   Provides RPC and metrics endpoints

**Ports:**

-   `9119`: RPC server
-   `9000`: Prometheus metrics

#### 2. Lit Actions Server (`lit-server`)

An HTTP server that:

-   Executes Lit Protocol actions
-   Integrates with Lit PKPs for decentralized signing
-   Provides signing services to the relay

**Ports:**

-   `3050`: HTTP API

### Communication Flow

```
User Request → Relay (port 9119)
                  ↓
            Lit Server (port 3050) → Lit Protocol Network
                  ↓
            Sign with PKP
                  ↓
            Relay → Blockchain (Base, Optimism, etc.)
```

### Docker Networking

Both services run on the `porto-network` bridge network, allowing them to communicate using service names:

-   Relay calls lit-server via `http://lit-server:3050`
-   Health checks use `localhost` within each container

## Troubleshooting

### Service Won't Start

**Issue:** Container exits immediately

**Check:**

1. Review logs: `docker-compose logs relay`
2. Verify all environment variables in `.env`
3. Ensure `relay.yaml` has valid configuration
4. Check that private keys have `0x` prefix

### Health Check Failing

**Issue:** Container shows as "unhealthy"

**Check:**

1. Wait 40-60 seconds for services to fully initialize
2. Check logs for startup errors
3. Verify ports aren't already in use: `lsof -i :9119` and `lsof -i :3050`

### Can't Connect to Chains

**Issue:** Relay can't connect to RPC endpoints

**Check:**

1. Verify RPC URLs in `relay.yaml` are correct
2. Test RPC endpoints manually with `curl`
3. Check if you need authentication/API keys
4. Ensure WebSocket endpoints use `wss://` not `https://`

### Lit Server Connection Failed

**Issue:** Relay can't reach lit-server

**Check:**

1. Verify lit-server is healthy: `docker ps`
2. Check the endpoint in `relay.yaml` is `http://lit-server:3050/execute` (not `localhost`)
3. Ensure both containers are on the same network: `docker network inspect porto_porto-network`

### Invalid PKP Configuration

**Issue:** Lit actions fail to execute

**Check:**

1. Verify `PKP_PUBLIC_KEY` is correct (130 hex characters)
2. Ensure `PKP_WALLET_OWNER_PRIVATE_KEY` is valid
3. Confirm PKP is registered on datil-test network
4. Verify PKP is added as SIGNER in crowd liquidity

### Transaction Signing Failures

**Issue:** Transactions fail to sign or submit

**Check:**

1. Verify `RELAY_MNEMONIC` is valid BIP39
2. Check signer accounts have sufficient gas funds
3. Ensure `RELAY_SETTLER_SIGNER_KEY` matches the settler private key in `relay.yaml`
4. Verify contract addresses are correct

### Rebuilding Services

If you make configuration changes:

```bash
# Rebuild and restart
docker-compose down
docker-compose up --build

# Force rebuild without cache
docker-compose build --no-cache
docker-compose up
```

### View Detailed Logs

Enable debug logging:

```bash
# In .env file
RUST_LOG=debug

# Or override when running
RUST_LOG=debug docker-compose up
```

### Common Error Messages

**"InvalidSender" in logs:**

-   The sender address isn't whitelisted in your contracts
-   Verify contract configurations match deployed addresses

**"Connection refused to lit-server":**

-   Check `relay.yaml` uses `http://lit-server:3050` not `http://localhost:3050`

**"insufficient funds for gas":**

-   Fund your signer accounts with native tokens (ETH)

**"failed to connect to RPC":**

-   Verify RPC endpoints are accessible
-   Check API keys are valid
-   Ensure you're using WebSocket URLs (`wss://`) where needed

### Getting Help

If you encounter issues:

1. Check logs: `docker-compose logs -f`
2. Review configuration files for typos
3. Verify all deployment steps were completed
4. Ensure test funds are available in all accounts
5. Consult the main [README.md](./README.md) for additional configuration options

## Next Steps

After successfully running the relay:

1. **Test Transactions**: Send test transactions through the relay
2. **Monitor Metrics**: Set up Prometheus/Grafana for the metrics endpoint
3. **Add More Chains**: Extend `relay.yaml` with additional chains
4. **Production Deployment**: Review security best practices and deploy to production infrastructure
5. **Scale**: Consider running multiple relay instances behind a load balancer

For more advanced configuration options, see the main [README.md](./README.md) and [relay.example.yaml](./relay.example.yaml).
