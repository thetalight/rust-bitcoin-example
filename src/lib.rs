use bitcoin::{Address, Network};

pub mod p2pkh;
pub mod p2wpkh;

const BTC_TEST_URL: &str = "https://blockstream.info/testnet/api";

// Just for test, DO NOT USE!!!
const PRIVATE_KEY_BYTES: [u8; 32] = [
    216, 166, 206, 234, 67, 115, 17, 206, 67, 244, 2, 74, 142, 138, 59, 3, 118, 156, 69, 148, 111,
    104, 216, 47, 49, 253, 0, 104, 186, 79, 60, 224,
];

fn receivers_p2wpkh_address() -> Address {
    "tb1qxhk9f09sx6xk9ct3gaz3kcy3vgl9ydnsqpwttc"
        .parse::<Address<_>>()
        .expect("a valid address")
        .require_network(Network::Testnet)
        .expect("valid address for mainnet")
}
