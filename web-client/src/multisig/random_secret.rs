use nimiq_serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_derive::TryFromJsValue;

/// A random secret that proves a {@link Commitment} for signing multisignature transactions.
/// It is supposed to be kept secret (similar to a private key).
#[derive(TryFromJsValue)]
#[wasm_bindgen]
#[derive(Clone)]
pub struct RandomSecret {
    inner: nimiq_keys::multisig::commitment::Nonce,
}

#[wasm_bindgen]
impl RandomSecret {
    #[wasm_bindgen(getter = SIZE)]
    pub fn size() -> usize {
        nimiq_keys::multisig::commitment::Nonce::SIZE
    }

    #[wasm_bindgen(getter = serializedSize)]
    pub fn serialized_size(&self) -> usize {
        RandomSecret::size()
    }

    /// Parses a random secret from a {@link RandomSecret} instance, a hex string representation, or a byte array.
    ///
    /// Throws when a RandomSecret cannot be parsed from the argument.
    #[wasm_bindgen(js_name = fromAny)]
    pub fn from_any(secret: &RandomSecretAnyType) -> Result<RandomSecret, JsError> {
        let js_value: &JsValue = secret.unchecked_ref();

        if let Ok(secret) = RandomSecret::try_from(js_value) {
            return Ok(secret);
        }

        if let Ok(string) = serde_wasm_bindgen::from_value::<String>(js_value.to_owned()) {
            Ok(RandomSecret::from_hex(&string)?)
        } else if let Ok(bytes) = serde_wasm_bindgen::from_value::<Vec<u8>>(js_value.to_owned()) {
            Ok(RandomSecret::deserialize(&bytes)?)
        } else {
            Err(JsError::new("Could not parse random secret"))
        }
    }

    /// Deserializes a random secret from a byte array.
    ///
    /// Throws when the byte array contains less than 32 bytes.
    pub fn deserialize(bytes: &[u8]) -> Result<RandomSecret, JsError> {
        let secret = nimiq_keys::multisig::commitment::Nonce::deserialize_from_vec(bytes)?;
        Ok(RandomSecret::from(secret))
    }

    /// Creates a new random secret from a byte array.
    ///
    /// Throws when the byte array is not exactly 32 bytes long.
    #[wasm_bindgen(constructor)]
    pub fn new(bytes: &[u8]) -> Result<RandomSecret, JsError> {
        Self::deserialize(bytes)
    }

    /// Serializes the random secret to a byte array.
    pub fn serialize(&self) -> Vec<u8> {
        self.inner.serialize_to_vec()
    }

    /// Parses a random secret from its hex representation.
    ///
    /// Throws when the string is not valid hex format or when it represents less than 32 bytes.
    #[wasm_bindgen(js_name = fromHex)]
    pub fn from_hex(hex: &str) -> Result<RandomSecret, JsError> {
        let bytes = hex::decode(hex)?;
        Self::deserialize(&bytes)
    }

    /// Formats the random secret into a hex string.
    #[wasm_bindgen(js_name = toHex)]
    pub fn to_hex(&self) -> String {
        hex::encode(self.serialize())
    }

    /// Returns if this random secret is equal to the other random secret.
    pub fn equals(&self, other: &RandomSecret) -> bool {
        self.inner == other.inner
    }
}

impl From<nimiq_keys::multisig::commitment::Nonce> for RandomSecret {
    fn from(secret: nimiq_keys::multisig::commitment::Nonce) -> RandomSecret {
        RandomSecret { inner: secret }
    }
}

impl RandomSecret {
    pub fn native_ref(&self) -> &nimiq_keys::multisig::commitment::Nonce {
        &self.inner
    }

    pub fn take_native(self) -> nimiq_keys::multisig::commitment::Nonce {
        self.inner
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "RandomSecret | string | Uint8Array")]
    pub type RandomSecretAnyType;
}
