# DD Blind Box - CosmWasm Smart Contract

A secure and feature-rich CosmWasm smart contract for blind box/NFT minting with decentralized voting and fair reward distribution.

## Features

### 🎁 Blind Box System
- **Multi-scale Support**: Tiny (10), Small (100), Medium (1K), Large (10K), Huge (100K) NFT collections
- **Sequential Minting**: Automatic token ID assignment with overflow protection
- **Base Token Configuration**: Flexible base token denomination support

### 🗳️ Decentralized Voting
- **Commit-Reveal Scheme**: Secure voting with cryptographic commitments
- **Time Window Validation**: Configurable commit, reveal, and closed windows
- **State Machine**: Robust voting state transitions (Commit → Reveal → Closed → Commit)
- **Duplicate Protection**: Prevents duplicate votes and commitments

### 🏆 Fair Reward Distribution
- **Three-Tier System**: 10% (2x multiplier), 50% (1x), 40% (0.5x) reward distribution
- **Secure Random Selection**: Uses multiple entropy sources for fair participant selection
- **DoS Protection**: Maximum voter limits to prevent gas exhaustion attacks

### 🔒 Security Features
- **Access Control**: Owner-only administrative functions
- **Reentrancy Protection**: State updates before external calls
- **Input Validation**: Comprehensive parameter validation and bounds checking
- **Pause Mechanism**: Emergency pause functionality for all operations

### 📜 CW721 NFT Standard
- **Full Compliance**: Complete CW721 standard implementation
- **Transfer & Approval**: Standard NFT transfer and approval mechanisms
- **Query Support**: Comprehensive query interface for all NFT operations
- **Metadata Support**: Token URI and metadata query capabilities

## Architecture

### Core Components

1. **Contract State**
   - `Config`: Global configuration (owner, total supply, base token, voting state)
   - `DEPOSITS`: User deposit tracking with principal amounts
   - `COMMITS`: Voting commitments storage
   - `REVEALS`: Revealed votes with salt values
   - `TIERS`: Final reward tier assignments
   - `TOKENS`: NFT token ownership and metadata
   - `OPERATORS`: Global operator approvals

2. **Voting Flow**
   ```
   Commit Phase → Reveal Phase → Closed Phase → Finalize
   ```

3. **Reward Distribution**
   ```
   Random Selection → Tier Assignment → Payout Calculation → Bank Transfer
   ```

## Usage

### Instantiation

```rust
let msg = InstantiateMsg {
    scale: Scale::Medium,  // 1,000 NFTs
    base: Coin::new(1000, "ujunox"),  // Base token amount
};
```

### Voting Process

1. **Commit Phase**: Users submit cryptographic commitments
2. **Reveal Phase**: Users reveal their votes with salt values
3. **Closed Phase**: Voting is closed, ready for finalization
4. **Finalization**: Owner triggers reward distribution

### NFT Operations

```rust
// Deposit to mint NFT
let msg = ExecuteMsg::Deposit {};

// Transfer NFT
let msg = ExecuteMsg::TransferNft {
    recipient: "recipient_address".to_string(),
    token_id: 1,
};

// Approve NFT
let msg = ExecuteMsg::Approve {
    spender: "spender_address".to_string(),
    token_id: 1,
};
```

## Security Considerations

### Implemented Protections

- **Permission Control**: All administrative functions require owner privileges
- **Reentrancy Guards**: State updates completed before external calls
- **Input Validation**: Comprehensive validation of all user inputs
- **DoS Prevention**: Maximum voter limits and gas optimization
- **Window Validation**: Time-based operation restrictions
- **State Machine**: Enforced voting state transitions

### Audit Recommendations

- Regular security audits for voting mechanisms
- Monitor for new attack vectors in commit-reveal schemes
- Validate random number generation entropy sources
- Review reward distribution algorithms for fairness

## Testing

The contract includes comprehensive test coverage:

- **Unit Tests**: 112 tests covering all functionality
- **Integration Tests**: End-to-end voting and reward distribution
- **Edge Cases**: Boundary conditions and error scenarios
- **Security Tests**: Access control and input validation

Run tests:
```bash
cargo test
```

## Dependencies

- `cosmwasm-std`: CosmWasm standard library
- `cw-storage-plus`: Enhanced storage utilities
- `dd_algorithms_lib`: Decentralized decision algorithms
- `sha2`: Cryptographic hashing for commitments
- `hex`: Hexadecimal encoding/decoding

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please ensure all tests pass and follow the existing code style.

## Deployment

The contract is optimized for CosmWasm deployment with:
- Optimized binary size
- Gas-efficient operations
- Standard CosmWasm interface compliance

For deployment instructions, see the `scripts/` directory.
