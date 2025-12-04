#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bitcoin::{
        Address, Amount, CompressedPublicKey, OutPoint, PrivateKey, ScriptBuf, Sequence,
        Transaction, TxIn, TxOut, Txid, WPubkeyHash, Witness,
        absolute::LockTime,
        hashes::Hash,
        key::Secp256k1,
        secp256k1::Message,
        sighash::{EcdsaSighashType, SighashCache},
        transaction::Version,
    };
    use reqwest::Client;
    use serde::{Deserialize, Serialize};

    use crate::{BTC_TEST_URL, PRIVATE_KEY_BYTES};

    #[derive(Debug, Deserialize, Serialize)]
    struct Utxo {
        txid: String,
        vout: u32,
        value: u64,
    }

    #[tokio::test]
    async fn it_works() {
        let privkey =
            PrivateKey::from_slice(&PRIVATE_KEY_BYTES, bitcoin::Network::Bitcoin).unwrap();
        let secp = Secp256k1::signing_only();
        let pubkey = privkey.public_key(&secp);
        // let wpkh = pubkey.wpubkey_hash().expect("key is compressed");
        let addr = Address::p2wpkh(
            &CompressedPublicKey(pubkey.inner),
            bitcoin::Network::Testnet,
        );
        println!("Address: {:?}", addr);

        let client = Client::new();
        let utxo_url = format!("{}/address/{}/utxo", BTC_TEST_URL, addr);
        let utxos: Vec<Utxo> = client
            .get(&utxo_url)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let utxo = utxos.get(0).unwrap();

        let txin = TxIn {
            previous_output: bitcoin::OutPoint {
                txid: Txid::from_str(&utxo.txid).unwrap(),
                vout: utxo.vout,
            },
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new(),
        };

        // println!("value :{:?}", utxo.value);

        let fee = 500;
        let txout = TxOut {
            value: Amount::from_sat(utxo.value - fee),
            script_pubkey: addr.script_pubkey(), // send to the same address
        };

        let mut raw_tx = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: vec![txin],
            output: vec![txout],
        };

        let input_index = 0;

        let mut cash = SighashCache::new(&mut raw_tx);
        let sig_hash = cash
            .p2wpkh_signature_hash(
                input_index,
                &addr.script_pubkey(),
                Amount::from_sat(utxo.value),
                EcdsaSighashType::All,
            )
            .unwrap();
        println!("sig_hash:{:?}", sig_hash.to_string());
        let msg = Message::from(sig_hash);
        let signature = secp.sign_ecdsa(&msg, &privkey.inner);

        let signature = bitcoin::ecdsa::Signature {
            signature,
            sighash_type: EcdsaSighashType::All,
        };

        *cash.witness_mut(input_index).unwrap() = Witness::p2wpkh(&signature, &pubkey.inner);
    }
}
