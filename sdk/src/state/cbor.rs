use std::convert::TryFrom;

use anyhow::{anyhow, Result};
use cid::multihash::Code;
use cid::Cid;
use fvm_ipld_blockstore::Block;
use fvm_ipld_encoding::CborStore;

use crate::state::error::Error::{InvalidCid, MismatchedCid, PutFailed};

/// A blockstore that delegates to IPLD syscalls.
pub struct CborBlockstore;

// TODO: Don't hard-code the size. Unfortunately, there's no good way to get it from the
//  codec at the moment.
pub const SIZE: u32 = 32;

impl fvm_ipld_blockstore::Blockstore for CborBlockstore {
    fn get(&self, cid: &Cid) -> Result<Option<Vec<u8>>> {
        // If this fails, the _CID_ is invalid. I.e., we have a bug.
        fvm_sdk::ipld::get(cid)
            .map(Some)
            .map_err(|e| InvalidCid(e, *cid).into())
    }

    fn put_keyed(&self, k: &Cid, block: &[u8]) -> Result<()> {
        let code = Code::try_from(k.hash().code()).map_err(|e| anyhow!(e.to_string()))?;
        let k2 = self.put(code, &Block::new(k.codec(), block))?;
        if k != &k2 {
            return Err(MismatchedCid(*k, k2).into());
        }
        Ok(())
    }

    fn put<D>(&self, code: Code, block: &Block<D>) -> Result<Cid>
    where
        D: AsRef<[u8]>,
    {
        let k = fvm_sdk::ipld::put(code.into(), SIZE, block.codec, block.data.as_ref())
            .map_err(PutFailed)?;
        Ok(k)
    }
}

impl CborStore for CborBlockstore {}
