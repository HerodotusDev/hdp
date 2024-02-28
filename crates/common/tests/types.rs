use common::types::{
    hex_to_8_byte_chunks_little_endian, split_hex_into_key_parts, Account, Header, HeaderProof,
    MPTProof, Uint256,
};

#[test]
fn cairo_format_header() {
    let original_header = Header{
        rlp: "f90226a018a6770e7e502f9209082c676922bbf1ad4f984924a17743d3044e6b3ffd8f19a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347947cbd790123255d9467d22baa806c9f059e558dc1a0156be497b45c06194d49508c8dca1ecef038ab4d3bd6060de6cfa2c9a4c3591ca0dcf5dc08c6e2720af2576fad9b9cccc66c0b50e53ebdd946bf0529ea750acb27a0d365f953867eadc22b2b2ded7cd620d92214e06671fd95e4f4d0b4747a4d2906b901000040020a0900000206083c210411006001d1080000040000001800a48100083040001000e00102090013424000844400000004004800020030144004a0600820448001000821811080002108880408100000404001140a1000004c004080020a280280280a108000025800044a044903800914004080000000c04015980109800022000002018804242400200a004a00000000201208804808001000c652088103080400100000060c00000000001000100022800a18000a2034a200040200010000013e000030000510000020020401004001100088000052008e0345802b0828b0005000a0011201022002808420402401000020001000820022400840081080834b90248401c9c380838ef3b5846588daac856c696e7578a03310d07ba1b9123c44429746f84d32df7e725178ae2c66404a3afad502c0a402880000000000000000849ac020c3a01e922a1e8e795414af0458d9af8d1fa08f5365cb4efb05273c3004b882cd3c84".to_string(),
        proof: HeaderProof{
            leaf_idx: 56993,
            mmr_path: vec!["0x4f582f7c3e936d25c2979f6c473278c17fb4c1cc02b5dc27b8226d41135fc9c".to_string()]
        }
    };

    let result = hex_to_8_byte_chunks_little_endian(&original_header.rlp);
    assert_eq!(
        result.chunks,
        vec![
            "0xe77a618a02602f9",
            "0x672c0809922f507e",
            "0x49984fadf1bb2269",
            "0x6b4e04d34377a124",
            "0x4dcc1da0198ffd3f",
            "0xb585ab7a5dc7dee8",
            "0x4512d31ad4ccb667",
            "0x42a1f013748a941b",
            "0xbd7c944793d440fd",
            "0xd267945d25230179",
            "0x559e059f6c80aa2b",
            "0xb497e46b15a0c18d",
            "0x8d8c50494d19065c",
            "0x3b4dab38f0ce1eca",
            "0xa4c9a2cfe60d06d6",
            "0x8dcf5dca01c59c3",
            "0xad6f57f20a72e2c6",
            "0xe5500b6cc6cc9c9b",
            "0xea2905bf46d9bd3e",
            "0xf965d3a027cb0a75",
            "0x2d2b2bc2ad7e8653",
            "0xe01422d920d67ced",
            "0xb4d0f4e495fd7166",
            "0x1b906294d7a74",
            "0x20000090a024000",
            "0x60001104213c0806",
            "0x4000008d101",
            "0x30080081a4001800",
            "0x90201e000100040",
            "0x44840040421300",
            "0x2004800040000",
            "0x200860a004401430",
            "0x1081210800018044",
            "0x1008048808210080",
            "0x100a140140400000",
            "0xa028040004c0000",
            "0x80100a28800228",
            "0x349044a04005802",
            "0x804000140980",
            "0x800901981540c000",
            "0x488010200002200",
            "0x4a000a20002424",
            "0x4880081220000000",
            "0x810852c600100008",
            "0x600001000040803",
            "0x1000000000000c",
            "0xa00180a80220010",
            "0x100020400a23420",
            "0x3000003e010000",
            "0x104022000001005",
            "0x880010014000",
            "0x82b8045038e0052",
            "0x1201a0005000b028",
            "0x4020848002200201",
            "0x10002000000124",
            "0x1008400840220082",
            "0xc9018424904b8380",
            "0x6584b5f38e8380c3",
            "0x756e696c85acda88",
            "0xb9a17bd01033a078",
            "0x4df8469742443c12",
            "0x2cae7851727edf32",
            "0xc002d5fa3a4a4066",
            "0x8802a4",
            "0xc320c09a84000000",
            "0x54798e1e2a921ea0",
            "0x1f8dafd95804af14",
            "0x5fb4ecb65538fa0",
            "0x3ccd82b804303c27",
            "0x84"
        ]
    );

    assert_eq!(result.chunks_len, 553)
}

#[test]
fn cairo_format_account() {
    let original_account = Account {
        address: "0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4".to_string(),
        account_key: "0x4ee516ed41ff168cfccb34c4efa2db7e4f369c363cf9480dc12886f2b6fb82a5".to_string(),
        proofs: vec![
            MPTProof {
                block_number: 4952100,
                proof: vec![
                  "0xf90211a0b260487078406bef4549faf0cf3b8689f38e132759b79f1b38f921bb0770725ba0f07c30281fed0e6c948d671b34a5c17926f19abaf816de01df4df13b2f21da0fa0841f208d25bb1a776e3c2171b6480a0c724d3dfda547b2640a44cc588e070b0ea06d0c3d0c26e0d6445ecb5b6a33fcf689223e7730ea633c10b3da596b95e0f1b2a00bf1fd1401f66489c80505a599832504e7ac4b21aa9c9d112773bcfc27092457a0954a14a3720833725342d37906ae2deea600f972e3be67bdfb9b2d9edb6c1591a041272f02efe0185e5efff8a544069f9e4a95ea84529bc19f1e4c2eeb1eac570aa060ca91f51e2cf585b712149c5a445bfac9538702f0d3426bee60aefa4c87ecbfa0cdba627f2918168968ec9b36b4de19bd296d5ac150c1a9b0f9c5fe16d2124cdda02d3691c76b294a895c7d87f1d6fdc9277f2535aadc78490e0d079a3e35659f1ea0bda5248f65638b83926e03c0d51a48620f41350e586285f895bbe1b22dfed40ea0758672f5a0bf7dcc32d1f39bacf1cc3e91a1152d3b8fefe0196b0c84e951a3ada0270bab342f3d7067df0a85afd721081abe2f9ba381e11c4d7696e3ea8374e781a0ebe3b5f3fce25e553b5d58f6fe8e1ca2bb755c36bb676fadcd2cafad96bcb33da047b0214744d7948e4006726fd5d457b1f161c39009214f65eb8294b334e6892ca01220abfad74e06651e36c1cc9f8c4f969bc6405f427808b0b59a8a536e95861780".to_string(),
                  "0xf90211a0a954661b9074f2aaa86f30dd72090c661033ea23bd53d5f4f81868f52fa36244a01796443bb911da9cf09b45140ced3c3b29a7ec3a718d064048309e080c181794a0c985eb0a0e53c4b2263b82e139bb99ab8734afe19f83a7c97d5fb5b5fc06a99ba0d68c13a7b9e4b19c3ef8c9c1ff28e79e6aa010f8c6d35732fb4ce94e48f414e9a02f4562fc142d982ff694d94cb140f320bee079f647061a89d867678cdca6dc9fa08c5cf6bec1e86b386077b2a601054b68c0e16da54f30ec4f997ff867072ff8d2a063f36f3b4d020c5662868715f4a794d12b9e35f2fe0839b0f53b5ca8b34026cea0958229737a3c3b26d0f7724a5f8b9c3f4aa198b8a5df96cd145eed5a1731f406a086f0a5107a6285864856d06c9fe42a852c9820ca3ecd114215a339ed12b9a97fa085670a6d72ee17acfc1a19550001596930bcfd8c5c52f18960bda455c39feef1a0cf03b39cf61db940a99e655a2ce1c9d8658e42d0f7340d414a8cdcadf7abb1bea07e61ec23442705596ce9886a93ed7d827478189654982650619160630fa68119a002eab370c4f7a0075bc748d614d2642402672b47b441a402dc0d941ebc962d30a06c9bc270052090f0ede819cd4858012659e2b12bb76400f7f501afa19ca38f25a0abb7bda7ae949eb2d0a8aca07f02edcb45cade684078b75ada26f76e1d574c73a045ba9374b98c8f1336f76608f91f4e984dab978dac5447d3fdbc0d1fb43c694e80".to_string(),
                  "0xf90211a0310581eee06b354207627fa84f8b7446bea7a72d57cace6c728d1e858ccfe63aa0a7a547cafa9c44fdfb89cc146d8c7eb24426186563caebef1f5221a57174f00ca0253fa6bbcebbbd65a7f42d7df002e806116f357a883a138e50d500421c3ec949a01cbaee38a8329511ea697563a730a82c18d25f2fd8084a3bfae6bd7104260c8da01b5a60992ab62791cae894a6f7a49c08199ee555de60e005f7d1f75fe4d6de6ba0a6ab5ded2187ff4c89752775bbadb7a053c5bcf3dc979f447bc0290961729aeaa0de9b27b305f034f73f618f572e67a05acc8bddc5fa94c07157d20fa988129704a0d546a721d807a2d15da202a60b3abaa40952a9da4f64aa09b27bc28dae814afba05d7cc04107956e1b0da8bfec0addcbe0ddecbb24f53b58a0fe4b009a3bae0b3fa0833db77f8fe4133aae2f0239cb6e3da14cd882f412caf4c70f1ef32e9a2d3232a0dab5929f9adabf64c7e37c545c1bb85c8d4d8ddee3a8357c698312abce225581a0d19b61dcbee08cef44ea3561e4338c725f8a10c105010a4ec5339565277251a9a077c2f9761ff69043b057da256b511a480d36ddd9114daa50d58ea0eb545307c9a0006475af74e7e0f09f64d153f07e942778d4ef0b1f0782733f5bee2a3fb6cd20a0aa1929dd092358f3579f527dd23b1b8920b2275c5a7dfefe53ae4f1fc7c175f5a03d4d663b89b086103333453e892cfdfd53213c5f68b2be25ff6fcf601cbb953180".to_string(),
                  "0xf90211a01313d4d949f1c6d765b380c85167c36ddd4a50a1a3a5632206807c5fbb84b8eaa0017174652da6eeb6b1ced2348401322a4ae8a64958e0985f464d1f6cec54ac25a0b7afe535d15307c31d59e46ab82a46d088da1fb28c6b8fbd401ad0f00ba2b10aa0294d97b0f709fcd8217b194674c3bb1e72f1ac8a72157a9ddee31c3acc0d6c32a0653cd435f791aa09ed6d90792a44b3a8204be0ff7af542e5982a784a6c6dcd67a0eb0e5f713b28a0b7dab330813d618d171a2fbea462b8eb48988f9988b0622b26a04c560f056b670c4369f09d48cfe5b64abdd71ec834270859e7d6919833ec0461a02cf718dd9f2f1e81b14d486fa0f2aa147b53a0c221179a2337a4f417ae8d63bda0987e4ef0bc43132f4fc8656df6d3a46c8f455c4001bbce8aa95b76fbf60eb388a061fa7b1646e130b0834e828c17097e4fa8795688ebc9e72bb557bc2a2854d2eaa03dc26feadef3e60cb1b32ae3ca20985d2fa7fda872157083287ff47c9d9a5438a0f0d76fc7d874c2ed155faa3f1f4b752cd24461e4eb17fe47d050acc107d9f317a0092abbbc3bdc3a9bb5c5d8e748cfb5054b72c03af1604e30caf7e97950dde889a0ddf1f5166efd8afda29144efe533bd454180247277067acac325501a454fa3e8a0dc601283681eb2a758b9271efa8a63eaae1df82613d45759cf425cfc54c840b6a0b6984c5746a144a9560b1df1723fb8aba697c17804b1ef01583cb51fc88dc30c80".to_string(),
                  "0xf90211a0e270c90cb49f72d776b8b15fbc4c1af480f1dd535d77896f624af8c2988b3607a0e2352b3689f57a39b887fb660e044159234423e620a59be04a55149e0c66b1cda04ff631d4515ec8577655836e399d596cfffa3bf5b2dcf75c96cf547853369d8ca0ede6659948cdb9a039c37a5ffb9a61c19635a6f668773cb51742d68bb46ddeffa05fac9e7f4107ab42660f707eec0880374ff8c1b050d7703c117a1cfe7decdff8a01f7c71dea2af699e513ca602e4663a8637637eedbf705b4e70e1763145f3c297a0fa0a75e221c480ad81a80efdf7f5861c4bf0b83421c50ce4d700192ebef6fb77a0c12ff9a99d8fcd3a7c69623a0cde48200024f13fea08b18e4b6d9fba8646f2c7a02e92386cbf8d5f7b2650836ba619505bf8bf501f99b9e64a103ff44b5f142e01a00de2306019f2caf07ae25444e6654a8354cc7247ef75abfac7701e1275e83e6ea0f25b7cd356d3da6922bc3f761e78118f9b19e0e610fc00381715695e5b757275a09fb473b88b52d4f9c0f705faa6ba416c0b42d60a0a20a248157febf1adf59f7fa028ee2e741ba400aae27205e3232e26137940f5d63f51a145745cae1b14aa3508a0c5843b9d96bca482fed2314387363c03025943deb30397335e88fe585b3d6c65a089968fa3749452a13d5e4b1b4c92330f0f21ac68123c8704e06675c03e21e3a5a031c1f51dc4d740a7d25e55e660f7063fb2f0583c74df7413f2240d4b02c3958f80".to_string(),
                  "0xf901918080a05ad8f58c5eeb611212582513105c3967f2be0c437c163e0a500f3e99376a1d0d8080a07039ebe73cd8e4a725451e6a9353314ec4b7a0899b79c192404bae1f13d8b8f9a0423c1e5c930fa044f5cfb134d6beffab09fb41d294a297394799ae06574b72c8a020a06be73b79a2293abc6e08b110a2dfe44804f705ff5c1220c170c927515b19a079fbc4003101e9c9b417f7cc4ca45621de54380d7302cf4ce1f2030cfacdf6eaa0cf60aebe174d15925f304a8226f6162b32a4f817f3153c9db02b359f95496bd6a0ce698f9fed05871cd65245e01ec8a4d1e50361b0e1b51468a93f12f19ce8ba6da01ec3512523b23d000cfd0ff5eaf77787223de5fc55dd3cda8d3792634a037977a08bc0a97065188cd5310ebceb9a17200651e6c767c47f047735404284c09b8c58a0864ff5e9ad3b6a76602e5575c52f9ec123679f7509451fa98cb2e152081a21b5a0a6d5f12320ebfce67e1d97ff2ff6cd21b8e83a97fb39460496828dfbfaa5361ea0045d68895435f0e949bced85f319c36da7e6a2ea3fd5ff45632284f26ddfe79a80".to_string(),
                  "0xf851808080808080808080a07b3c71cc818328815c79bcd344c717789bde929b23bf30bfe28a36ca3cad72cd80808080a0b0e04f694ef56d458c3c2fa191aa95862a20a393419a2f1fbe009e75864c662d8080".to_string(),
                  "0xf8719d3d41ff168cfccb34c4efa2db7e4f369c363cf9480dc12886f2b6fb82a5b851f84f821a78890242aa8ffb4eba0bc4a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a0c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470".to_string()
                ]
              }]
    };

    let result = hex_to_8_byte_chunks_little_endian(&original_account.address);
    assert_eq!(
        result.chunks,
        vec!["0xaad30603936f2c7f", "0x12f5986a6c3a6b73", "0xd43640f7"]
    );

    let proof_result = hex_to_8_byte_chunks_little_endian(&original_account.proofs[0].proof[0]);
    assert_eq!(proof_result.chunks_len, 532);
    assert_eq!(
        proof_result.chunks,
        vec![
            "0x704860b2a01102f9",
            "0xf0fa4945ef6b4078",
            "0x27138ef389863bcf",
            "0xbb21f9381b9fb759",
            "0x307cf0a05b727007",
            "0x678d946c0eed1f28",
            "0x9af12679c1a5341b",
            "0xf14ddf01de16f8ba",
            "0x1f84a00fda212f3b",
            "0x3c6e771abb258d20",
            "0x4d720c0a48b67121",
            "0x440a64b247a5fd3d",
            "0x6da00e0b078e58cc",
            "0x5e44d6e0260c3d0c",
            "0x2289f6fc336a5bcb",
            "0xb3103c63ea30773e",
            "0xa0b2f1e0956b59da",
            "0x8964f60114fdf10b",
            "0x4258399a50505c8",
            "0x119d9caa214bace7",
            "0x57240927fcbc7327",
            "0x330872a3144a95a0",
            "0x2dae0679d3425372",
            "0x67bee372f900a6ee",
            "0x156cdb9e2d9bfbbd",
            "0xe0ef022f2741a091",
            "0x644a5f8ff5e5e18",
            "0x9b5284ea954a9e9f",
            "0xac1eeb2e4c1e9fc1",
            "0x1ef591ca60a00a57",
            "0x5a9c1412b785f52c",
            "0xf0028753c9fa5b44",
            "0x4cfaae60ee6b42d3",
            "0x7f62bacda0bfec87",
            "0x369bec6889161829",
            "0xc15a6d29bd19deb4",
            "0x16fec5f9b0a9c150",
            "0x91362da0dd4c12d2",
            "0x877d5c894a296bc7",
            "0x35257f27c9fdd6f1",
            "0x9a070d0e4978dcaa",
            "0xa5bda01e9f65353e",
            "0x6e92838b63658f24",
            "0x410f62481ad5c003",
            "0xbb95f88562580e35",
            "0x75a00ed4fe2db2e1",
            "0x32cc7dbfa0f57286",
            "0x913eccf1ac9bf3d1",
            "0x19e0ef8f3b2d15a1",
            "0xa0ada351e9840c6b",
            "0x67703d2f34ab0b27",
            "0x1a0821d7af850adf",
            "0x4d1ce181a39b2fbe",
            "0x81e77483eae39676",
            "0x5ee2fcf3b5e3eba0",
            "0x1c8efef6585d3b55",
            "0x6f67bb365c75bba2",
            "0xb3bc96adaf2ccdad",
            "0xd7444721b047a03d",
            "0xd4d56f7206408e94",
            "0x210990c361f1b157",
            "0xe634b39482eb654f",
            "0xd7faab2012a02c89",
            "0x9fccc1361e65064e",
            "0x425f40c69b964f8c",
            "0x6e538a9ab5b00878",
            "0x80178695"
        ]
    );

    let account_key_result = split_hex_into_key_parts(&original_account.account_key);
    assert_eq!(
        account_key_result,
        Uint256 {
            low: "0x7edba2efc434cbfc8c16ff41ed16e54e".to_string(),
            high: "0xa582fbb6f28628c10d48f93c369c364f".to_string()
        }
    )
}