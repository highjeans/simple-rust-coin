use ed25519_dalek::{SigningKey, VerifyingKey};
use keyring::Entry;
use rand::rngs::OsRng;

struct KeyPair {
    private_key: SigningKey,
    public_key: VerifyingKey
}

fn generate_new_private_key() -> SigningKey {
    SigningKey::generate(&mut OsRng)
}

fn get_existing_private_key() -> Result<SigningKey, Box<dyn std::error::Error>> {
    let entry = Entry::new("simple-rust-coin", "private_key")?;
    let password = entry.get_password()?;
    let key_bytes: &[u8] = &hex::decode(password)?;
    let mut private_key_arr: [u8; 32] = [0; 32];
    private_key_arr.copy_from_slice(key_bytes);
    Ok(SigningKey::from_bytes(&private_key_arr))
}

fn get_keys() -> KeyPair {
    let private_key = match get_existing_private_key() {
        Ok(key) => {key}
        Err(_) => {
            // Couldn't find saved key, generate a new one (existing coins are now lost)
            generate_new_private_key()
        }
    };

    let public_key = private_key.verifying_key(); // a private key will only have 1 public key

    KeyPair {
        private_key,
        public_key
    }
}

struct Wallet {
    key_pair: KeyPair
}