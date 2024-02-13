use std::collections::HashMap;

use super::memory::RlpEncodedValue;

pub fn get_example_headers() -> HashMap<u64, RlpEncodedValue> {
    let mut headers = HashMap::new();
    headers.insert(10399990, "f90266a045adb684cb5458019c496206c1383894c360fe969a1028ba44955eadfa585cc5a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d4934794b636a68f834b4d75af9edc5fb0138bb4758ed293a01db2388923f7c78680b4a46bae725637013d74ad787ec5c861d3ade3df882d81a093586eb5f2781ded334a2a03d178f41dc06f271d7f1ff429e4da6ef42d12a773a0361590775fea7857cc048b9324c03e96f287199803ce1440ff1e12c5c6008049b901000420000a200308000025201005a30400008962800402185dc600144280040082221400010101200458002b0d88008028004206808408400402108f0812246200240a204365100109051c082a020081204200001060440090044044448100082100028001060640c011401a802000090331000408243804009402201240802082820403801141050a4a00208283202050000f10058894008000411050512800220a200000042275800280894080000202460040030000408001ce00282400000002a8c24210000200014a30040015020b04800020608800000850440240c06100011002000000200988001800000880128a050400329081c144080a040800000480839eb0f68401c9c380836f9a8e8465aa87809f496c6c756d696e61746520446d6f63726174697a6520447374726962757465a0c653e1c1cee990147f4439776cc3ead6f175e081998c33c93da41653112e89ce8800000000000000000da039db3f9d1fe0756e5aef4e2f0241ad957e999e49c981809c018425d0080f6cd2830400008405320000a0713ce910d12e99ba96492ff2f6411d4e0a3e567ab419e92e60cf5fc4aa74db7a".to_string());
    headers.insert(10399991, "f90261a02ef5bd5264f472d821fb950241aa2bbe83f885fea086b4f58fccb9c9b948adcfa01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d493479494750381be1aba0504c666ee1db118f68f0780d4a01fa9a7ad0deb9e4c0762aed5a02a0eb0be12864363eb2f6434ad55454352533ea0e4be7e5ed75566d7ab90c1610d81a8f38f3b56b3c338f211646ae32aebd4e690a0f8973cc47d1dac1a9f71fec14ec19dcde2ad1e6c6b6336e42b1da0322a6c7581b901007432022a09014808001c000b91ab000c408a444206011a6200994018c004101a06d1c003460021145888092814458890000a020092b4e4c0023a3710332c01329252440a050520886880061f020882214f91000e2a64a50a8024444c832446718206009dc60020a0a7e158a104002800080884d0041085084aa83010012ae2c88210a39401140080b04100004b200700902184c3ea95a1c8401532f944820040276e5b884c9b3290120009408b0604a5a401281a41ec508000c2886b280a488a2208d0222800a801080622b00a7602091c2000430481101002800c32400862623110400100400219bc010708a1008080320416001a88804109ce05041097041080839eb0f78401c9c38084012ad0678465aa879899d883010d0a846765746888676f312e32312e36856c696e7578a04d3c8cca805e07a07e9cc74a0b5891629f26bec5ed54b74867ff0f77bcc994b38800000000000000000da002a352293855b89564efc797e98f05368f9d42d0a99be7dadf6d45165eaaf0f0830600008405300000a0f8dbbd84390d2bf3b34c8f581d42e08dd907764e46972aad56cac48cbb650473".to_string());
    headers.insert(10399992, "f9025da0000f7814e0558a13469620a059eff4510844d8129d2bd0f814bcbae62adff515a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d4934794455e5aa18469bc6ccef49594645666c587a3a71ba06ef1804556d1e997392ae8ee19e4abf930fed4ecd7199f6f28e5450cae49835da0e96e520730416715728f451bc12e7e348d08c6dc84e42716833dc36b0c6b49c4a0624b8ee1e6e66f17ae42630abeb8fd1cc69c91b3d9bb9c83d6ef2376f8620dc2b9010004000008890100000204208001a10000400210440400004808040410000410420050006000002000180013001440840000020800806040200010051002240210800205004440208000002209402000098200100020042108001004040500002080020000320000800061408000d008090800440004100000482a0010014420008020830401840000800000000f000000000100004210208041051224430000008300008144021000080109e00004042524810800010028200400802a20200000420080060800a00100032010041203081000000200802200000004301010620101100000000010080c000300028804004000000012888000002800002085020080839eb0f88401c9c380836fba078465aa87a499d883010d0a846765746888676f312e32312e36856c696e7578a05cdb41adf931d8a929aaf09f6a825be65513d809e167144bcbec60eabf7922a58800000000000000000ea0a4ad1832a56ef60e6b49323c6ce4221e5524d11aca2c35623b324d094b5541f0808405300000a0bb3eb95012cfbeea1e030903f186a1df253e26cc6f0dd5495b677fefabb73c30".to_string());
    headers.insert(10399993, "f90261a0a4928d67f0c66d5c62a44d2c180badca94ea2aa3bf91f25f97d5a5039d9b812ea01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d493479462c8bcec31584187f69295294fbb7620d2e0cd72a01b9b14e361177b5fcde1e8b1cb95b07dccd1bcab35157958e1b5bc4a4b1ad9e3a09897276962157a75b5ddf8b44cd46c1821e36c4fc733ad9706f8f3a14d2778bea066378d7d59d4d3800f512077e63e054274841411de3909a25aeb098bbc342fb1b90100842100082001400010142e0a8ba10100400820420002083004b110100004102286902224048461001800016884042088806078049220420406001318462400301012265a850e008044000a0a800082204481000032c6011110408400cb20682480020002061010d00148408020100801181885c0461406005008001600002040821442040004804050400042038001414402840b423480c8401310f040800000ca000680940030506b0889400104060b96910a0011c04008018600c3140049004208c16220014001000220b0185600080040401006a000100000080210086200ad1048008a1480088c00000020008e02000010009a88015044800240c48cc00480839eb0f98401c9c380836fea688465aa87b09ad983010d09846765746889676f312e32302e3133856c696e7578a0d5c0c49915e6584d0749c52b49716a3fb4b552a0e837253b037498584686036a8800000000000000000ea00ba5704b8908ebfc357a7d939e111750e5e489d7480e8f8f086bf7cc9a256bd4830c000084052a0000a072071503b19e199e9dceaffce80a9320b2dfea37d069b37c49224edd32ba5932".to_string());
    headers.insert(10399994, "f90260a089a45b58b24fb782ee50cf5b8d116934acaec3e67527713d33cfb114f1736dcfa01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d4934794f36f155486299ecaff2d4f5160ed5114c1f66000a09d2cd16c62f80864a2599183efd95156b58791ebf7464594385daeb202e57dcfa015fb8fd8d2d91227211208562f9bbc905c6cd09be95187ff7f455d5b472e6bcda0899e91c5a0c5082e5275f11d5c7ae263d39b605cd0023e249177572d4088c247b901000621180a20014ac0601c004681a1105c91d844c22600185307110510060c119206908402064021405c0019a804448ba22842c67092a0634103109700522c6432104e442ae40d80d96910025e028c80a146010001b26c818580048684912548608116008886760a9036409ac080920801081884e824100548543a52b4c08e30c582b203840941215030673130435001918401de03ef1d40c8504572706022116003a014ec65283e1003008942880084212e09080201c0408201c1486f6880d8015218d042341a0a50004223b00c570209060060220088431001c0080640026240511c4000080000698901d2b8a840801c2a0154185ec8c04148c800141004021080839eb0fa8401c9c38083fe4eaf8465aa87d499d883010d0a846765746888676f312e32312e36856c696e7578a05625df0bccfd09cb146fc3a90fd7e8befbb1c8476ad0ad7345638c93538f65d78800000000000000000ea06e3d4e738fdb774f4c872d5187168f26df7d71a3086b35692d26c6a3bf7ce734830600008405300000a05ca81e9c05ea8b4ccd7a21aaf7aacec158f256d601d16e40839c60d98afa7fc0".to_string());
    headers.insert(10399995, "f9025da0c69a0d458b1ea41c72d0a9a43f45cd947e681e7a7c33e1dc3a0fd8abc0f82d0fa01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d4934794455e5aa18469bc6ccef49594645666c587a3a71ba0094793b132c386d1ca31f83204a5ffa2ccef4edcc3fbffef39153aa52a1f31a0a0dee897271be94192b76eb37c54417c529f095ff1f32f6cbbe4695641855d3feea0456bd3352507c5f327f04f9c63244569af5d17ab5b576693e868a309c32cc54ab901000400000a30014a8000140242012100088000080100028800021000180404000a009000000c100100180041000004008000008004920041000020110002a0403014020042000420084410000b00000008442101082048251000002600010002340402010402000280004100a400001880280884c04014000104002094040820008a16021000000100104610104b0004080001010040108080504120105004000006a00000041010100020894010000010260a000001a040400081004a048040800208902a00000010040230200842000802200000c08000000040001200002200191004200200044822010002a0008204000000105a800004000800000004120080839eb0fb8401c9c38083efc6ff8465aa87e099d883010d0a846765746888676f312e32312e36856c696e7578a0147c52f02f979f18d2b3bb9ef4c149f0c08183dbc6c3f7787f431d0f77e2a20e8800000000000000000fa0adda430f291ebd8b29300dc152eb01614a3e12a54ec2e85fb6f2b0e8862d95a2808405300000a0ac815c7a815300e1c4d8c3a82287d225cb6a2e590dc1f8a3e32a6b2637b312b8".to_string());
    headers.insert(10399996, "f90260a036508f57fc618c0eb97f575dea291b029c8fe7dabda8ccadb50f67bc6dc7f6c8a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d4934794000095e79eac4d76aab57cb2c1f091d553b36ca0a01d7a5880ff6993159108cda8d8d001fd3c60d21b43901dcb3196b14cfd3d3ebba016b549e2cfe2da964f3ca90c02b85e76263cb7057f9121462cbb7914f8e8a96fa0e840f3d545f932d5bda765791bc958439b382623f7c6dc4db8918b9574e4306fb901000000000800004aa100100002000100084088000000001c400290011800040022028000000000010008002800000420842802800012000100001015001020403412128000600408104110000a8000820044210000004001009008000400000a20000210042700001000420080000408000a0800c0203004000403001000082040a212000000000100104010380000000000010082401400c05041a00040800000028000400020100006009002000000000000008001004080008100222080800002080022000002000002202008d2020906000000048802000040820204002200001200000000004100020030a0000004020000184a080001100800000005028080839eb0fc8401c9c38083b9bf718465aa87ec99d883010d09846765746888676f312e32312e35856c696e7578a032bf691f17cfacbdf561efe601596f5ecda83dd1e3561ab6ef20dbc88d6bfd3a88000000000000000010a0c439b85dd1dc29e603e6dea16ae1608e69ea660122e67f9a138d994dfb849f7b830c000084052a0000a036b203eace2145f2548866b1800b04bb7fbd0aac31d4f221149f9d0e98d6fd21".to_string());
    headers.insert(10399997, "f90251a0ab7d0c58b9c65dfe639c2f211419ae348a5addf750dee4251e333d05b9963e4ca01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d4934794c6e2459991bfe27cca6d86722f35da23a1e4cb97a0111aba04a60a5c9b12a8ffdb2c25c559a07bff9bcd1fca0def062505ba20c814a07fc3a0fa37591d62f85ef4a6d41c603a28bf406c7b9f9708981641489833358ba02163dad73e78112d9b95112d0586deb63cdd6011b035db341673c997fef149eab901001523000a00114ec00014040ab121000c00880002040018640210011a810c0002969090060404214258002dcd00148080a142c050920663040030170052a4603010120018c40c20994258121fa220a2614f010108a35c201096008604912042780002000586040081024018a04082d882bc088cda401004184e02f314020820808212022083000108104010150380014180013c93e01482e850413150c004022017a01080148110120b408d42800400292411490203f440800dc5026b2180db804388c0020a00ec00090220b50856434806400008b08810100850480244006300251042210a008409a8050080a0008824228014d05e8880410e880c0400e6220080839eb0fd8401c9c38083eb6b288465aa88048a4e65746865726d696e64a0bf9288229d7fcae357296a6625ab4544e09bf51bd06038931bf7c50e443df6f788000000000000000010a05636ee17cad0dcfe4b825748afa13954cd23ab282c1b8a1ea3f3182452bc6a0b830a00008405300000a055bc73c49ce77e6b083e0aaf908ffa7232de633942a89c2f2155b80b7999a68c".to_string());
    headers.insert(10399998, "f90266a0453e6e9c59cc79225484522372d91892467b92f93de4b8f959fa103c4d66fd48a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d4934794b636a68f834b4d75af9edc5fb0138bb4758ed293a0e18ae94924ff9b4e00d89c68c6b1877ab5fcb42ec624cab0854bad2d25bb864fa0b22db3a12150f67635470138f8310d4ae2b730f25991abde493ca0e313ddac90a06d98daccfe6d2d8864d8695efad9ba54d90007ff01102314412a4048b308eaebb901000600000820234a800834040a0721004880100440c42208494214041a42141022009030450d4011001e0403a0408400880428c000924651008200130092244030310a02420d040220541200ab0020840044118000216c80100400064881010222080200044222008890543084024048000108c4c06030040405002090409820028216520200000500326810100340000040109480001004c010456011450001000288040434201810000089c00003010007100001a080400280a10046248040002208c00240000204000a61200842204806440020c0800001004040024042600001b810004d00001808010000a480801c004040107a860001800100010886022480839eb0fe8401c9c3808353073d8465aa88109f496c6c756d696e61746520446d6f63726174697a6520447374726962757465a01546662902dd3009dc02026e83490ef770422479b77a2956d77447cf04115fd188000000000000000011a02e543617a0f520e23d7c36e89cbcceccead71bfe74bfc01c2038c4c59e252923830400008405340000a0cf2322ccda5e2a9bd8dd40c25158ef5b07e2a681ddce97ebe7201793a66d98e8".to_string());
    headers.insert(10399999, "f90260a0c8d36d24d81f2e36a0f8c1f36f8add24dc8714320a78b17d07b25424e097ffa2a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d4934794f97e180c050e5ab072211ad2c213eb5aee4df134a019f55a7f1fe370a77267d5c3327474ab2071802f7038a825a79c4bf78544c579a04eb5a4126fc6adc987334d3cb30a0785acf1dc1ca0019b5416288f839184060ea07274312f7366d6e50308a6bf1b2534e9eef27fc1f905125aa7b5f6d7625192deb901000422000d48014a88021400028165080800fc41402a04184203d194180484105242d004620200014018001b20144480808042a040b0f041600a101750122c4238907206004415208c4850221a802a88214e018400326c61088c9c041581060262c006a08c16088098a468408012000c21581804c8049005004e2a5010092a20808e13d31401802120384012114f000000e84124056a15a0e85145e360468200408388809c44203300022189604000042404100000032840a000818822608000008208d0023c012801000622a00c520208170000000080601000c0e42218106ac111340020000020690c000328a2804004020004105aa800510c6840001405029080839eb0ff8401c9c380838ee43e8465aa882899d883010d0a846765746888676f312e32312e36856c696e7578a053b87a12a76bb28a417cbda3acc6b241d7d849717140be043b8c1a2f8cb41ecb88000000000000000010a09c34780aa89b7419c9e7f57494d29e8c557e64d0cf04c3db57092eee73e7af00830a00008405320000a0b30e828e1ecadc3c7b194c582d1cc0896eeb2493025282153e4fe0df7419e589".to_string());
    headers.insert(10400000, "f90263a0792521e7510ba898aff41504d3ef2214619b81bcd731c38b2ee010228eebaabfa01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d4934794b64a30399f7f6b0c154c2e7af0a3ec7b0a5b131aa00aa14102830b3bb9ef07b0ed181062ed9ffc671d60b53eedb6d86ad85d3d4f50a0bc85c0c84815c05fd4f6d7139e592064716addb4e1a79e3322d7f08238768969a070806f936670ae9d136239961c487f752467c451ff1709af94e5254128bdaf14b901000460020e880b4a81007400038725009990884202a741984a4211011a84a4111216904c000700010068200ba80906b9c814228215925641400a129780122c44b0b17a080c4605a0a14854103e060898a84605001a3664406c060825c4a12642200084048d062c02853440588031012a00006a94c9281885000622d014080a2086863b821e25042500124490104302010058851485ac1d00c89255327009ea004553a2200a6420b2140a408940820004080670001112c0448004c1c96320a64a0a0288d2f21e240a40080e2bb41b571f081708c0004490e01020c04d034854624011104029004400198800130aa200c27462b214107ac0804104c840152811021480839eb1008401c9c3808386481f8465aa88349f496c6c756d696e61746520446d6f63726174697a6520447374726962757465a00fd71f682d35bd1cb9c06bd2e6c637a79bb5cb25715ef90031c3c4cae99acfad88000000000000000010a0c8b02416e58787efe88ec75d1bae2a4f5ca2adea51b15477700d835bb1923871808405360000a047b14fc563202a2e1c1e14c1f15da41f74ffdefaca5e65a4f138875938074032".to_string());
    headers
}

pub fn get_example_accounts() -> HashMap<u64, HashMap<String, RlpEncodedValue>> {
    let mut accounts = HashMap::new();
    let mut account_10399990 = HashMap::new();
    account_10399990.insert(
        "0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6".to_string(),
        "f8440180a01c35dfde2b62d99d3a74fda76446b60962c4656814bdd7815eb6e5b8be1e7185a0cd4f25236fff0ccac15e82bf4581beb08e95e1b5ba89de6031c75893cd91245c".to_string(),
    );
    let mut account_10399991 = HashMap::new();
    account_10399991.insert(
        "0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6".to_string(),
        "f8440280a01c35dfde2b62d99d3a74fda76446b60962c4656814bdd7815eb6e5b8be1e7185a0cd4f25236fff0ccac15e82bf4581beb08e95e1b5ba89de6031c75893cd91245c".to_string(),
    );
    let mut account_10399992 = HashMap::new();
    account_10399992.insert(
        "0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6".to_string(),
        "f8440201a01c35dfde2b62d99d3a74fda76446b60962c4656814bdd7815eb6e5b8be1e7185a0cd4f25236fff0ccac15e82bf4581beb08e95e1b5ba89de6031c75893cd91245c".to_string(),
    );
    accounts.insert(10399990, account_10399990);
    accounts.insert(10399991, account_10399991);
    accounts.insert(10399992, account_10399992);
    accounts
}

pub fn get_example_storages() -> HashMap<u64, HashMap<String, HashMap<String, String>>> {
    let mut storages = HashMap::new();
    let mut storage_10399990 = HashMap::new();
    storage_10399990.insert("0x00000000000000adc04c56bf30ac9d3c0aaf14dc".to_string(), {
        let mut storage = HashMap::new();
        storage.insert(
            "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            "0x0000000000000000000000000000000000000000000000000000000000000001".to_string(),
        );
        storage
    });
    let mut storage_10399991 = HashMap::new();
    storage_10399991.insert("0x00000000000000adc04c56bf30ac9d3c0aaf14dc".to_string(), {
        let mut storage = HashMap::new();
        storage.insert(
            "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            "0x0000000000000000000000000000000000000000000000000000000000000002".to_string(),
        );
        storage
    });
    let mut storage_10399992 = HashMap::new();
    storage_10399992.insert("0x00000000000000adc04c56bf30ac9d3c0aaf14dc".to_string(), {
        let mut storage = HashMap::new();
        storage.insert(
            "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            "0x0000000000000000000000000000000000000000000000000000000000000001".to_string(),
        );
        storage
    });
    storages.insert(10399990, storage_10399990);
    storages.insert(10399991, storage_10399991);
    storages.insert(10399992, storage_10399992);
    storages
}
