use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use vaultrs::{
    api::{
        pki::{
            requests::GenerateCertificateRequestBuilder, responses::GenerateCertificateResponse,
        },
        transit::requests::VerifySignedDataRequest,
    },
    client::VaultClient,
    error::ClientError,
    pki::cert,
    transit,
};

const TRANSIT_MOUNT: &str = "transit";
const TRANSIT_KEY: &str = "nxlog-agent-manager";

pub async fn generate_certificate(
    client: &VaultClient,
    mount: &str,
    role: &str,
) -> Result<GenerateCertificateResponse, ClientError> {
    let mut builder = GenerateCertificateRequestBuilder::default();
    let &mut _ = builder
        .common_name("common_name")
        .format("pem")
        .private_key_format("pkcs8");

    cert::generate(client, mount, role, Some(&mut builder)).await
}

pub async fn sign(client: &VaultClient, data: &[u8]) -> Result<String, ClientError> {
    let resp = transit::data::sign(
        client,
        TRANSIT_MOUNT,
        TRANSIT_KEY,
        &BASE64.encode(data),
        None,
    )
    .await?;

    Ok(BASE64.encode(resp.signature.as_bytes()))
}

pub async fn verify(
    client: &VaultClient,
    data: &[u8],
    signature: &str,
) -> Result<bool, ClientError> {
    let signature = BASE64.decode(signature).unwrap();
    let signature = String::from_utf8(signature).unwrap();

    let verify_resp = transit::data::verify(
        client,
        TRANSIT_MOUNT,
        TRANSIT_KEY,
        &BASE64.encode(data),
        Some(VerifySignedDataRequest::builder().signature(signature)),
    )
    .await?;

    Ok(verify_resp.valid)
}
