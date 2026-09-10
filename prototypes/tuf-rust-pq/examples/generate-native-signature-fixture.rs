use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use aws_lc_rs::digest;
use sequoia_openpgp as openpgp;
use serde::Serialize;

use openpgp::cert::prelude::*;
use openpgp::policy::StandardPolicy;
use openpgp::serialize::stream::{Armorer, Message, Signer};
use openpgp::serialize::SerializeInto;
use openpgp::types::HashAlgorithm;

const ARTIFACT: &[u8] = b"codiquary issue 20 fixture\n";
const ARTIFACT_SHA256: &str = "d13ecc865c37b23650615038c232eccfa5770c8bc345dbe98db595273fecda4a";
const ARTIFACT_SHA512: &str =
    "01177f64b7deeb816e99718bd235288d429718e2dd6a4d1e2aee833d456e5ec8dd1224a5f9cd61c03629a596a3cb2154f55d8c732c4ff3bdc2b5d5b7b6df46a2";
const PUBLIC_CERTIFICATE_NAME: &str = "synthetic-signer-public.asc";
const DETACHED_SIGNATURE_NAME: &str = "artifact.sig.asc";
const MANIFEST_NAME: &str = "manifest.json";

#[derive(Serialize)]
struct Manifest {
    schema: &'static str,
    identity: Identity,
    signer: SignerIdentity,
    input: InputIdentity,
    outputs: Vec<OutputIdentity>,
}

#[derive(Serialize)]
struct Identity {
    classification: &'static str,
    purpose: &'static str,
    authority: &'static str,
}

#[derive(Serialize)]
struct SignerIdentity {
    fingerprint: String,
    algorithm: &'static str,
}

#[derive(Serialize)]
struct InputIdentity {
    literal_utf8: &'static str,
    byte_length: usize,
    sha256: String,
    sha512: String,
}

#[derive(Serialize)]
struct OutputIdentity {
    name: &'static str,
    byte_length: usize,
    sha256: String,
    sha512: String,
}

fn main() -> openpgp::Result<()> {
    let output_dir = parse_output_directory()?;

    let artifact_sha256 = sha256_hex(ARTIFACT);
    let artifact_sha512 = sha512_hex(ARTIFACT);
    if artifact_sha256 != ARTIFACT_SHA256 || artifact_sha512 != ARTIFACT_SHA512 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "the embedded artifact does not match its accepted hashes",
        )
        .into());
    }

    let (secret_cert, revocation) = CertBuilder::new()
        .set_cipher_suite(CipherSuite::RSA2k)
        .add_userid(
            "Codiquary synthetic nonproduction native-signature fixture <fixture@synthetic.invalid>",
        )
        .add_signing_subkey()
        .generate()?;
    drop(revocation);

    let policy = &StandardPolicy::new();
    let signing_keypair = secret_cert
        .keys()
        .unencrypted_secret()
        .with_policy(policy, None)
        .supported()
        .alive()
        .revoked(false)
        .for_signing()
        .next()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "generated certificate has no usable signing key",
            )
        })?
        .key()
        .clone()
        .into_keypair()?;

    let mut detached_signature = Vec::new();
    {
        let message = Message::new(&mut detached_signature);
        let message = Armorer::new(message)
            .kind(openpgp::armor::Kind::Signature)
            .build()?;
        let mut signer = Signer::new(message, signing_keypair)?
            .detached()
            .hash_algo(HashAlgorithm::SHA256)?
            .build()?;
        signer.write_all(ARTIFACT)?;
        signer.finalize()?;
    }

    let fingerprint = secret_cert.fingerprint().to_hex();
    let public_cert = secret_cert.strip_secret_key_material();
    if public_cert.is_tsk() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "secret key material remained after stripping",
        )
        .into());
    }

    let public_certificate = public_cert.armored().to_vec()?;
    if !public_certificate.starts_with(b"-----BEGIN PGP PUBLIC KEY BLOCK-----") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "public certificate serialization produced unexpected armor",
        )
        .into());
    }
    if !detached_signature.starts_with(b"-----BEGIN PGP SIGNATURE-----") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "detached signature serialization produced unexpected armor",
        )
        .into());
    }

    let manifest = Manifest {
        schema: "codiquary.synthetic-native-signature-fixture.v1",
        identity: Identity {
            classification: "synthetic/nonproduction",
            purpose: "native signature verification fixture only",
            authority: "not a deployment, package, trust-store, or TUF authority",
        },
        signer: SignerIdentity {
            fingerprint,
            algorithm: "OpenPGP RSA-2048 detached signature with SHA-256",
        },
        input: InputIdentity {
            literal_utf8: "codiquary issue 20 fixture\n",
            byte_length: ARTIFACT.len(),
            sha256: artifact_sha256,
            sha512: artifact_sha512,
        },
        outputs: vec![
            output_identity(PUBLIC_CERTIFICATE_NAME, &public_certificate),
            output_identity(DETACHED_SIGNATURE_NAME, &detached_signature),
        ],
    };

    let mut manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
    manifest_bytes.push(b'\n');

    fs::create_dir(&output_dir)?;
    write_new(&output_dir, PUBLIC_CERTIFICATE_NAME, &public_certificate)?;
    write_new(&output_dir, DETACHED_SIGNATURE_NAME, &detached_signature)?;
    write_new(&output_dir, MANIFEST_NAME, &manifest_bytes)?;

    Ok(())
}

fn parse_output_directory() -> io::Result<PathBuf> {
    let mut arguments = env::args_os();
    let _program = arguments.next();
    let output_dir = arguments.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected exactly one fresh output-directory argument",
        )
    })?;
    if arguments.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "unexpected extra argument",
        ));
    }

    let output_dir = PathBuf::from(output_dir);
    if !output_dir.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "output directory must be an absolute path",
        ));
    }
    Ok(output_dir)
}

fn output_identity(name: &'static str, bytes: &[u8]) -> OutputIdentity {
    OutputIdentity {
        name,
        byte_length: bytes.len(),
        sha256: sha256_hex(bytes),
        sha512: sha512_hex(bytes),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex(digest::digest(&digest::SHA256, bytes).as_ref())
}

fn sha512_hex(bytes: &[u8]) -> String {
    hex(digest::digest(&digest::SHA512, bytes).as_ref())
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(DIGITS[(byte >> 4) as usize] as char);
        encoded.push(DIGITS[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn write_new(directory: &Path, name: &str, bytes: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(directory.join(name))?;
    file.write_all(bytes)?;
    file.sync_all()
}
