#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bitcoin::{
        Address, PrivateKey, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid, Witness,
        absolute::LockTime,
        block::Version,
        consensus::serialize,
        key::Secp256k1,
        script::PushBytesBuf,
        secp256k1::Message,
        sighash::{EcdsaSighashType, SighashCache},
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
        let addr = Address::p2pkh(&pubkey, bitcoin::Network::Testnet);
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

        println!("value :{:?}", utxo.value);

        let fee = 500;
        let txout = TxOut {
            value: utxo.value - fee,
            script_pubkey: addr.script_pubkey(), // send to the same address
        };

        let mut raw_tx = Transaction {
            version: Version::ONE.to_consensus(),
            lock_time: LockTime::ZERO,
            input: vec![txin],
            output: vec![txout],
        };

        let cash = SighashCache::new(&raw_tx);
        let sig_hash = cash
            .legacy_signature_hash(0, &addr.script_pubkey(), EcdsaSighashType::All.to_u32())
            .unwrap();
        println!("sig_hash:{:?}", sig_hash.to_string());
        let msg = Message::from_slice(&sig_hash[..]).unwrap();
        let sig = secp.sign_ecdsa(&msg, &privkey.inner);
        println!("sig:{:?}", hex::encode(&sig.serialize_compact()));
        let mut sig_der = sig.serialize_der().to_vec();
        println!("sig_der:{:?}", hex::encode(&sig_der));
        sig_der.push(EcdsaSighashType::All.to_u32() as u8);

        let mut sig_push = PushBytesBuf::with_capacity(sig_der.len());
        sig_push.extend_from_slice(&sig_der).unwrap();
        let script_sig = ScriptBuf::builder()
            .push_slice(&sig_push)
            .push_key(&pubkey)
            .into_script();

        raw_tx.input[0].script_sig = script_sig;

        let raw_tx_bytes = serialize(&raw_tx);
        let raw_tx = hex::encode(&raw_tx_bytes);

        let res = client
            .post(format!("{}/tx", BTC_TEST_URL))
            .body(raw_tx)
            .send()
            .await
            .unwrap();

        let txid = res.text().await.unwrap();
        println!("txid: {}", txid);
    }
}
