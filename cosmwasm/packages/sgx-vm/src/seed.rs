use enclave_ffi_types::{HealthCheckResult, INPUT_ENCRYPTED_SEED_SIZE, NEWLY_FORMED_DOUBLE_ENCRYPTED_SEED_SIZE, NEWLY_FORMED_SINGLE_ENCRYPTED_SEED_SIZE, SINGLE_ENCRYPTED_SEED_SIZE, NodeAuthResult, OUTPUT_ENCRYPTED_SEED_SIZE};
use sgx_types::*;
use log::{error, info, debug};

use crate::enclave::ENCLAVE_DOORBELL;

extern "C" {
    pub fn ecall_init_node(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
        master_key: *const u8,
        master_key_len: u32,
        encrypted_seed: *const u8,
        encrypted_seed_len: u32,
        api_key: *const u8,
        api_key_len: u32,
    ) -> sgx_status_t;

    pub fn ecall_init_bootstrap(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
        public_key: &mut [u8; 32],
        spid: *const u8,
        spid_len: u32,
        api_key: *const u8,
        api_key_len: u32,
    ) -> sgx_status_t;

    pub fn ecall_generate_registration_key(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
        public_key: &mut [u8; 32],
    ) -> sgx_status_t;

    /// Trigger a query method in a wasm contract
    pub fn ecall_health_check(
        eid: sgx_enclave_id_t,
        retval: *mut HealthCheckResult,
    ) -> sgx_status_t;
    pub fn ecall_get_genesis_seed(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
        pk: *const u8,
        pk_len: u32,
        seed: &mut [u8; SINGLE_ENCRYPTED_SEED_SIZE as usize],
    ) -> sgx_status_t;
}

pub fn untrusted_health_check() -> SgxResult<HealthCheckResult> {
    //info!("Initializing enclave..");

    // Bind the token to a local variable to ensure its
    // destructor runs in the end of the function
    let enclave_access_token = ENCLAVE_DOORBELL
        .get_access(1) // This can never be recursive
        .ok_or(sgx_status_t::SGX_ERROR_BUSY)?;
    let enclave = (*enclave_access_token)?;

    //debug!("Initialized enclave successfully!");

    let eid = enclave.geteid();
    let mut ret = HealthCheckResult::default();

    let status = unsafe { ecall_health_check(eid, &mut ret) };

    if status != sgx_status_t::SGX_SUCCESS {
        return Err(status);
    }

    Ok(ret)
}

pub fn untrusted_init_node(
    master_key: &[u8],
    encrypted_seed: &[u8],
    api_key: &[u8],
) -> SgxResult<()> {
    info!("Initializing enclave..");

    // Bind the token to a local variable to ensure its
    // destructor runs in the end of the function
    let enclave_access_token = ENCLAVE_DOORBELL
        .get_access(1) // This can never be recursive
        .ok_or(sgx_status_t::SGX_ERROR_BUSY)?;
    let enclave = (*enclave_access_token)?;

    info!("Initialized enclave successfully!");

    let eid = enclave.geteid();
    let mut ret = sgx_status_t::SGX_SUCCESS;

    let mut seed_to_enclave = [0u8; INPUT_ENCRYPTED_SEED_SIZE as usize];

    if (encrypted_seed.len()) > INPUT_ENCRYPTED_SEED_SIZE as usize {
        error!("Tried to setup node with seed that is too long");
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }

    match encrypted_seed.len() {
        NEWLY_FORMED_SINGLE_ENCRYPTED_SEED_SIZE => seed_to_enclave
            [0..NEWLY_FORMED_SINGLE_ENCRYPTED_SEED_SIZE]
            .copy_from_slice(encrypted_seed),
        NEWLY_FORMED_DOUBLE_ENCRYPTED_SEED_SIZE => seed_to_enclave
            [0..NEWLY_FORMED_DOUBLE_ENCRYPTED_SEED_SIZE]
            .copy_from_slice(encrypted_seed),
        _ => {
            error!("Received seed with wrong length");
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
    };

    let status = unsafe {
        ecall_init_node(
            eid,
            &mut ret,
            master_key.as_ptr(),
            master_key.len() as u32,
            seed_to_enclave.as_ptr(),
            seed_to_enclave.len() as u32,
            api_key.as_ptr(),
            api_key.len() as u32,
        )
    };

    if status != sgx_status_t::SGX_SUCCESS {
        return Err(status);
    }

    if ret != sgx_status_t::SGX_SUCCESS {
        return Err(ret);
    }

    Ok(())
}

pub fn untrusted_key_gen() -> SgxResult<[u8; 32]> {
    info!("Initializing enclave..");

    // Bind the token to a local variable to ensure its
    // destructor runs in the end of the function
    let enclave_access_token = ENCLAVE_DOORBELL
        .get_access(1) // This can never be recursive
        .ok_or(sgx_status_t::SGX_ERROR_BUSY)?;
    let enclave = (*enclave_access_token)?;

    info!("Initialized enclave successfully!");

    let eid = enclave.geteid();
    let mut retval = sgx_status_t::SGX_SUCCESS;
    let mut public_key = [0u8; 32];
    // let status = unsafe { ecall_get_encrypted_seed(eid, &mut retval, cert, cert_len, & mut seed) };
    let status = unsafe { ecall_generate_registration_key(eid, &mut retval, &mut public_key) };

    if status != sgx_status_t::SGX_SUCCESS {
        return Err(status);
    }

    if retval != sgx_status_t::SGX_SUCCESS {
        return Err(retval);
    }

    Ok(public_key)
}

pub fn untrusted_init_bootstrap(spid: &[u8], api_key: &[u8]) -> SgxResult<[u8; 32]> {
    info!("Hello from just before initializing - untrusted_init_bootstrap");

    // Bind the token to a local variable to ensure its
    // destructor runs in the end of the function
    let enclave_access_token = ENCLAVE_DOORBELL
        .get_access(1) // This can never be recursive
        .ok_or(sgx_status_t::SGX_ERROR_BUSY)?;
    let enclave = (*enclave_access_token)?;

    info!("Hello from just after initializing - untrusted_init_bootstrap");

    let eid = enclave.geteid();
    let mut retval = sgx_status_t::SGX_SUCCESS;
    let mut public_key = [0u8; 32];
    // let status = unsafe { ecall_get_encrypted_seed(eid, &mut retval, cert, cert_len, & mut seed) };
    let status = unsafe {
        ecall_init_bootstrap(
            eid,
            &mut retval,
            &mut public_key,
            spid.as_ptr(),
            spid.len() as u32,
            api_key.as_ptr(),
            api_key.len() as u32,
        )
    };

    if status != sgx_status_t::SGX_SUCCESS {
        return Err(status);
    }

    if retval != sgx_status_t::SGX_SUCCESS {
        return Err(retval);
    }

    Ok(public_key)
}

pub fn untrusted_get_encrypted_seed(
    cert: &[u8],
) -> SgxResult<Result<[u8; OUTPUT_ENCRYPTED_SEED_SIZE as usize], NodeAuthResult>> {
    // Bind the token to a local variable to ensure its
    // destructor runs in the end of the function
    let enclave_access_token = ENCLAVE_DOORBELL
        .get_access(1) // This can never be recursive
        .ok_or(sgx_status_t::SGX_ERROR_BUSY)?;
    let enclave = (*enclave_access_token)?;
    let eid = enclave.geteid();
    let mut retval = NodeAuthResult::Success;
    let mut seed = [0u8; OUTPUT_ENCRYPTED_SEED_SIZE as usize];
    let status = unsafe {
        crate::attestation::sgx::epid::ecall_legacy_verify_node_on_chain(
            eid,
            &mut retval,
            cert.as_ptr(),
            cert.len() as u32,
            &mut seed,
        )
    };

    if status != sgx_status_t::SGX_SUCCESS {
        return Err(status);
    }

    if retval != NodeAuthResult::Success {
        return Ok(Err(retval));
    }

    if seed.is_empty() {
        error!("Got empty seed from encryption");
        return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
    }

    Ok(Ok(seed))
}

pub fn untrusted_get_encrypted_genesis_seed(
    pk: &[u8],
) -> SgxResult<[u8; SINGLE_ENCRYPTED_SEED_SIZE as usize]> {
    // Bind the token to a local variable to ensure its
    // destructor runs in the end of the function
    let enclave_access_token = ENCLAVE_DOORBELL
        .get_access(1) // This can never be recursive
        .ok_or(sgx_status_t::SGX_ERROR_BUSY)?;
    let enclave = (*enclave_access_token)?;
    let eid = enclave.geteid();
    let mut retval = sgx_status_t::SGX_SUCCESS;

    let mut seed = [0u8; SINGLE_ENCRYPTED_SEED_SIZE as usize];
    let status = unsafe {
        ecall_get_genesis_seed(eid, &mut retval, pk.as_ptr(), pk.len() as u32, &mut seed)
    };

    if status != sgx_status_t::SGX_SUCCESS {
        debug!("Error from get genesis seed");
        return Err(status);
    }

    if retval != sgx_status_t::SGX_SUCCESS {
        debug!("Error from get genesis seed, bad NodeAuthResult");
        return Err(retval);
    }

    debug!("Done getting genesis seed, got seed: {:?}", seed);

    if seed.is_empty() {
        error!("Got empty seed from encryption");
        return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
    }

    Ok(seed)
}
