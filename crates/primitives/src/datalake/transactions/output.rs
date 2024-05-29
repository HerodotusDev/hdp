use serde::{Deserialize, Serialize};

use crate::datalake::output::{hex_to_8_byte_chunks_little_endian, CairoFormattedChunkResult};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct Transaction {
    pub key: String,
    pub block_number: u64,
    pub proof: Vec<String>,
}

impl Transaction {
    pub fn to_cairo_format(&self) -> TransactionFormatted {
        let key = self.key.clone();
        let proof_chunk_result: Vec<CairoFormattedChunkResult> = self
            .proof
            .iter()
            .map(|proof| hex_to_8_byte_chunks_little_endian(proof))
            .collect();

        let proof_bytes_len = proof_chunk_result.iter().map(|x| x.chunks_len).collect();
        let proof_result: Vec<Vec<String>> = proof_chunk_result
            .iter()
            .map(|x| x.chunks.clone())
            .collect();
        TransactionFormatted {
            key,
            block_number: self.block_number,
            proof_bytes_len,
            proof: proof_result,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct TransactionFormatted {
    pub key: String,
    pub block_number: u64,
    /// proof_bytes_len is the byte( 8 bit ) length from each proof string
    pub proof_bytes_len: Vec<u64>,
    pub proof: Vec<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct TransactionReceipt {
    pub key: String,
    pub block_number: u64,
    pub proof: Vec<String>,
}

impl TransactionReceipt {
    pub fn to_cairo_format(&self) -> TransactionReceiptFormatted {
        let key = self.key.clone();
        let proof_chunk_result: Vec<CairoFormattedChunkResult> = self
            .proof
            .iter()
            .map(|proof| hex_to_8_byte_chunks_little_endian(proof))
            .collect();

        let proof_bytes_len = proof_chunk_result.iter().map(|x| x.chunks_len).collect();
        let proof_result: Vec<Vec<String>> = proof_chunk_result
            .iter()
            .map(|x| x.chunks.clone())
            .collect();
        TransactionReceiptFormatted {
            key,
            block_number: self.block_number,
            proof_bytes_len,
            proof: proof_result,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct TransactionReceiptFormatted {
    pub key: String,
    pub block_number: u64,
    /// proof_bytes_len is the byte( 8 bit ) length from each proof string
    pub proof_bytes_len: Vec<u64>,
    pub proof: Vec<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_to_cairo_format() {
        let tx = Transaction {
            key: "0x3c".to_string(),
            block_number: 56089491,
            proof: vec![
                "0xf8f1a08cc9b883b97dba9b03d99eed8fd610e23d071548ec6d265f9894e36ba26d8d81a01169ff73ab2954ab11ac57b3cdf01078b06db293be44f6976efffea8b80fc89ca025fb87db0d4f7a77310a695f80563556c00ccc4f733e23b0e262d0227880fd53a03149b308e7d5be961643a0d97d063b20b4ae8ac838254f03bbfcd277791599dfa0fa80535271431021b3bc3a0d1260ea2788653b5baaeabcbda30fdcf7e8fb5762a023f963efe4d02cf9605fdefb699630abb3221ee5c85761ba35c71fff4fa394298080a01d5cb192e7df88bdb2d37f7004726c0dfe44e17418760965b251fd95de8da33a8080808080808080".to_string(),
                "0xf90211a05af791ace36809a869ff801b8f7c26289f9268e4f327602328dd03f987ad68bca0426f4626cb0eb2c1323a3f6d169fb795aa06e09f7a535ebe95c71fc286154594a03d9afe8228bbb1036fa5c94ea588a3e50540d28768f9d4ac49bdb7b1d7cb7c65a01e4203aadc267c7b80c94bf38998058f1c9f88b2d18c7126ca387232b5a64eeba0a7e3d546b410ca9cc0ea127ddeeff391ad6504b8890a933813d4ca759584a4faa042eac82fa090293825a285eade5cc06240171b5bf96089e014609fbd31684040a05cdeefec72269c31bfc15348ac6809153e7694be4621735c23c60e346d49a6c8a0eba86e6a025149616b0c8129434a4a47bb9e460bb3b94fbe3e85a82efadb4544a0e2a19e90409639fb4f84c5a34792a458ed91b54ed4c07891aece0e239ca604cea01dd25e91abc350e9bd2a4d00155ad973d8a50a038fc74cd5f4e72e7f4343cfe0a00a06a7fdeb963d919b31009c9df624cc14ab11514614abe5727f5edcd8f6da2fa05ed6086104237d7485e832f19e13124b54f2f4538250a452a1753c7e74cde6c5a03dfac53439a11d519cf006086e55ae270e7c758c7a76f1860807030ed79dbff2a041ba905221220902fbd90ff18ec4a13ea2899da616bb16b4daa7d16bb1fc0235a0fec2e7de248be2177ddebff7bf43f068559c731ca43e45367bbe4030543d3f96a01c585cd8ef1661dfaeb6a22733ca4fc29924c22cc34a8f3349293a5419fa81c380".to_string(),
                "0xf902df20b902db02f902d783aa36a7821d07843b9aca0085174876e80083a7d8c0940d71ee5b9e16db3f6e20121a68535df9433794af80b902645578ceae000000000000000000000000000000000000000000000000000000000020dc6d00000000000000000000000000000000000000000000000000000000000000a002b12c0d22350ef5f2e9cc145c22ede51b310c7facf8a6e20e8293f8807692a60597bbd6d17d83ca1db8a02068fb5082f36402ab14a38713e41a6bde901dab6d0800000000000011000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000d035e36a7acb3577ddfbdec56313e819499cc5bd8b07d655af032a49eeae42f3e034a27d4c5cd23960ca10e19c09ecc9540a0c6ea2eed3b613b021adb63fd4375000000000000000000000000000000000000000000000000000000000000d851074df775805ac3cfd3a03a20dabbe12379a53d59d24df288a5184f5b14c52004000504fa6e5eb930c0d8329d4a77d98391f2730dab8516600aeaf733a6123432000000000000000000000000000000000000000000000000000000000000000100000000000000000a0ed0804c39f746b7b50723f0d0ab8dae38ed0271baceca0000000000000000b3fdd8c33cdc38d0fb850127b407a4578b3d05e8ef431d960290cdc45b6c38d9029f864382703c009b80df07e4be21182c86f557a2cddff40000000000000000000000000000000004b93eff6363e8330682aac38870b56b000000000000000000000000000000000cbc138a3b24fd46745f971324dab1a600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000c001a0864ba6e66f7cdea987a236ea8d139e182b3d3cc145b39de943b6eae6569225e0a053397d49516e03947d7923cba3b42ad533519e56cf544dadc269030386e8d215".to_string()]
        };

        let tx_formatted = tx.to_cairo_format();

        assert_eq!(tx_formatted.key, "0x3c".to_string())
    }
}
