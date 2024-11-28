use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress};
use idb::{Database, Error, KeyPath, ObjectStore, TransactionMode};
use nimiq_bls::{G2Projective, LazyPublicKey, PublicKey};
use nimiq_serde::{Deserialize, Serialize};

/// Caches decompressed BlsPublicKeys in an IndexedDB
pub(crate) struct BlsCache {
    db: Option<Database>,
    keys: Vec<LazyPublicKey>,
}

#[derive(Serialize, Deserialize, Debug)]
struct BlsKeyEntry {
    compressed_key: String,
    public_key: String,
}

impl BlsCache {
    pub async fn new() -> Self {
        let db = match Database::builder("nimiq_client_cache")
            .version(1)
            .add_object_store(
                ObjectStore::builder("bls_keys")
                    .key_path(Some(KeyPath::new_single("compressed_key"))),
            )
            .build()
            .await
        {
            Ok(db) => Some(db),
            Err(err) => {
                log::warn!("idb: Couldn't create database {}", err);
                None
            }
        };

        BlsCache { db, keys: vec![] }
    }

    /// Add the given keys into IndexedDB
    pub async fn add_keys(&self, keys: Vec<LazyPublicKey>) -> Result<(), Error> {
        if let Some(db) = &self.db {
            let transaction = db.transaction(&["bls_keys"], TransactionMode::ReadWrite)?;
            let bls_keys_store = transaction.object_store("bls_keys")?;

            for key in keys {
                let mut result = Vec::new();
                key.uncompress()
                    .unwrap()
                    .public_key
                    .serialize_with_mode(&mut result, Compress::No)
                    .unwrap();
                let public_key = hex::encode(&result);
                let compressed_key = hex::encode(key.compressed().serialize_to_vec());

                let entry = BlsKeyEntry {
                    compressed_key,
                    public_key,
                };
                let entry_js_value = serde_wasm_bindgen::to_value(&entry).unwrap();
                bls_keys_store.put(&entry_js_value, None)?.await?;
            }
        }
        Ok(())
    }

    /// Fetches all bls keys from the IndexedDB and stores them, which makes the decompressed keys
    /// available in other places.
    pub async fn init(&mut self) -> Result<(), Error> {
        if let Some(db) = &self.db {
            let transaction = db.transaction(&["bls_keys"], TransactionMode::ReadOnly)?;
            let bls_keys_store = transaction.object_store("bls_keys")?;

            let js_keys = bls_keys_store.get_all(None, None)?.await?;

            for js_key in &js_keys {
                let value: BlsKeyEntry = serde_wasm_bindgen::from_value(js_key.clone()).unwrap();
                let public_key = PublicKey::new(
                    G2Projective::deserialize_uncompressed_unchecked(
                        &*hex::decode(value.public_key.clone()).unwrap(),
                    )
                    .unwrap(),
                );
                self.keys.push(LazyPublicKey::from(public_key));
            }
            transaction.await?;
        }
        Ok(())
    }
}
