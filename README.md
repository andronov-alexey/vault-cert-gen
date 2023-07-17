# vault-cert-gen
Vault  "500 Error" reproduction when generating 1000 RSA keys

## Prerequisites
### 1. Running Vault Docker Image ([official manual](https://hub.docker.com/r/hashicorp/vault)):
`docker run --cap-add=IPC_LOCK -e 'VAULT_DEV_ROOT_TOKEN_ID=root' -e 'VAULT_DEV_LISTEN_ADDRESS=0.0.0.0:8200' hashicorp/vault`
### 2. Enabled "PKI" engine with "cert-role-rsa" role with RSA-3072 key in Vault
### 3. `vault-cert-gen` binary from the current repo

## Run (default)
`RUST_LOG="warn,vault_cert_gen=trace,vaultrs=info" ./vault-cert-gen`

## Run (full command line)
`RUST_LOG="warn,vault_cert_gen=trace,vaultrs=info" ./vault-cert-gen --vault-addr http://localhost:8200 --vault-token root --vault-pki-mount pki --vault-pki-role cert-role-rsa --keys-count 1000`

## Run (customization: generate 2000 keys)  
`RUST_LOG="warn,vault_cert_gen=trace,vaultrs=info" ./vault-cert-gen --keys-count 2000`

## Help
`./vault-cert-gen --help`
