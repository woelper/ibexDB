#[derive(Serialize, Deserialize, Debug, Default)]
struct SyncBucket {
    key: String,
    value: String,
    herd: Vec<String>
}