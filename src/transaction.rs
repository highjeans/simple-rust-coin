use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct UTXO {
    pub amount: f64,
    pub owner: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub input_utxos: Vec<UTXO>,
    pub output_utxos: (UTXO, UTXO), // index 0: the UTXO gone to receiver, index 1: change
    pub signature: String
}

impl Transaction {
    pub fn new(input_utxos: &Vec<UTXO>, amount: f64, receiver: String, private_key: &SigningKey) -> Transaction{
        let mut input_amount = 0f64;
        let sender: String = input_utxos[0].owner.clone();
        for utxo in input_utxos {
            input_amount += utxo.amount;
        }
        let input_utxos = Vec::clone(input_utxos);
        let output_utxos = (
            UTXO {
                amount,
                owner: receiver.clone()
            },
            UTXO {
                amount: input_amount - amount,
                owner: sender.clone()
            }
        );

        let signature = hex::encode(private_key.sign((sender + &receiver + &amount.to_string()).as_ref()).to_bytes());

        Transaction {
            input_utxos,
            output_utxos,
            signature
        }
    }

    pub fn verify(&self) -> Result<(), Box<dyn std::error::Error>> {
        let sender = self.input_utxos[0].owner.clone();
        let receiver = self.output_utxos.0.owner.clone();
        let amount = self.output_utxos.0.amount;

        let sender_bytes: &[u8] = &hex::decode(sender.clone())?;

        let mut sender_arr: [u8; 32] = [0; 32];
        sender_arr.copy_from_slice(sender_bytes);

        let signature_bytes: &[u8] = &hex::decode(self.signature.clone())?;
        let mut signature_arr: [u8; 64] = [0; 64];
        signature_arr.copy_from_slice(signature_bytes);

        Ok(VerifyingKey::from_bytes(&sender_arr)?.verify((sender + &receiver + &amount.to_string()).as_ref(), &Signature::from_bytes(&signature_arr))?)
    }
}
