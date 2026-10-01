use aws_lc_rs::digest::{digest, SHA256};
use std::error::Error;
use std::path::Path;
use tough::{FilesystemTransport, RepositoryLoader};

const PUBLISHER_TARGET_BYTES: &[u8] =
    include_bytes!("data/publisher-client-binding/targets/artifact.bin");
const PUBLISHER_TARGET_LENGTH: u64 = 27;
const PUBLISHER_TARGET_SHA256: [u8; 32] = [
    0xd1, 0x3e, 0xcc, 0x86, 0x5c, 0x37, 0xb2, 0x36, 0x50, 0x61, 0x50, 0x38, 0xc2, 0x32, 0xec, 0xcf,
    0xa5, 0x77, 0x0c, 0x8b, 0xc3, 0x45, 0xdb, 0xe9, 0x8d, 0xb5, 0x95, 0x27, 0x3f, 0xec, 0xda, 0x4a,
];
const CLIENT_TARGET_NAME: &str = "artifact.bin";

#[tokio::test(flavor = "current_thread")]
async fn publisher_and_client_bind_exact_target_bytes(
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/publisher-client-binding");
    let root = std::fs::read(fixture.join("trusted-root.json"))?;
    let metadata = fixture.join("metadata").canonicalize()?;
    let targets = fixture.join("targets").canonicalize()?;
    let metadata_url = format!("file://{}/", metadata.display()).parse()?;
    let targets_url = format!("file://{}/", targets.display()).parse()?;

    let repository = RepositoryLoader::new(&root, metadata_url, targets_url)
        .transport(FilesystemTransport)
        .load()
        .await?;
    let mut client_targets = repository.all_targets();
    let (client_name, client_target) = client_targets
        .next()
        .ok_or("verified client metadata did not select artifact.bin")?;
    assert!(
        client_targets.next().is_none(),
        "verified client metadata must select exactly one target"
    );

    let publisher_sha256 = digest(&SHA256, PUBLISHER_TARGET_BYTES);
    assert_eq!(PUBLISHER_TARGET_BYTES, b"codiquary issue 20 fixture\n");
    assert_eq!(PUBLISHER_TARGET_BYTES.len() as u64, PUBLISHER_TARGET_LENGTH);
    assert_eq!(publisher_sha256.as_ref(), PUBLISHER_TARGET_SHA256);
    assert_eq!(client_name.raw(), CLIENT_TARGET_NAME);
    assert_eq!(client_target.length, PUBLISHER_TARGET_LENGTH);
    assert_eq!(
        client_target.hashes.sha256.as_ref(),
        publisher_sha256.as_ref()
    );

    Ok(())
}
