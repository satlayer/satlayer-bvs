// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.24;

import {Math} from "@openzeppelin/contracts/utils/math/Math.sol";

import {Test, console} from "forge-std/Test.sol";
import {MerkleProof} from "../src/MerkleProof.sol";

contract MerkleProofTest is Test {
    function test_verify_bbn() public pure {
        bytes32 leaf = keccak256(
            abi.encodePacked(
                keccak256(abi.encodePacked("bbn1eywhap4hwzd3lpwee4hgt2rh0rjlsq6dqck894100000000000000000"))
            )
        );

        bytes32[] memory proof = new bytes32[](1);
        proof[0] = bytes32(abi.encodePacked(hex"614a2406c3e74dca5a75c5429158a486c9d0d9eb5efbea928cc309beb6b3fce6"));

        bytes32 root = bytes32(abi.encodePacked(hex"4b83dc8ecaa7a9d69ac8a7c12718eed8639e1ba1a1b30a51741ccfd020255cec"));

        assertEq(MerkleProof.verify(proof, root, leaf, 1, 2), true);
    }

    function test_verify_evm() public pure {
        bytes32 leaf = keccak256(
            abi.encodePacked(
                keccak256(abi.encodePacked("0x86d6Fda2f439537da03a5b76D5aE26412F4c4235200000000000000000"))
            )
        );

        bytes32[] memory proof = new bytes32[](3);
        proof[0] = bytes32(0xc5d11bcf5b13a6839acbf0f57fe1b202fe159e5b5b3bbbd3b9dd1a69e1aa84dc);
        proof[1] = bytes32(0x8d25a6cb91e258d097872c7e37477e311da5fcd048037a7d729d9eac13903882);
        proof[2] = bytes32(0x8a08f27e959995b62300cc7b9cdebb565e9ba6c0bfabf76c58da0c98ac378e81);

        bytes32 root = bytes32(abi.encodePacked(hex"2016f97ae135385b6942e4aa35c97bdcfdd599c9ddcd750868f8366173d58d3c"));

        assertEq(MerkleProof.verify(proof, root, leaf, 3, 5), true);
    }

    function test_verify_complex() public pure {
        bytes32 leaf = keccak256(
            abi.encodePacked(keccak256(abi.encodePacked("bbn1j8k7l6m5n4o3p2q1r0s9t8u7v6w5x4y3z2a1b600000000000000000")))
        );

        bytes32[] memory proof = new bytes32[](4);
        proof[0] = bytes32(abi.encodePacked(hex"c99659b6e1c7a2df5c6ce352241ef43152f1fed170d2fdc8d67ee2c47d9f26a5"));
        proof[1] = bytes32(abi.encodePacked(hex"662362a44a545cd42cdf7e9c1cfc7eb3b55ebb3af452e1bd516d7329f0f490fa"));
        proof[2] = bytes32(abi.encodePacked(hex"8a12e22ffa7163c5249a71f3df41704c2e2ccf8bab02dec02fc8db2740f51256"));
        proof[3] = bytes32(abi.encodePacked(hex"afb5ee202bbe624a5d933b1eda40f5bf6bcd6674dbf1af8eea698ae023c104fe"));

        bytes32 root = bytes32(abi.encodePacked(hex"1c8dd9ca252d7eb9bf1cccb9ab587e9a1dccca4c7474bb8739c0e5218964a2b4"));

        assertEq(MerkleProof.verify(proof, root, leaf, 7, 9), true);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function test_revert_processProof_wrongTotalLeaves() public {
        bytes32 leaf = keccak256(
            abi.encodePacked(keccak256(abi.encodePacked("bbn1j8k7l6m5n4o3p2q1r0s9t8u7v6w5x4y3z2a1b600000000000000000")))
        );

        bytes32[] memory proof = new bytes32[](4);
        proof[0] = bytes32(abi.encodePacked(hex"c99659b6e1c7a2df5c6ce352241ef43152f1fed170d2fdc8d67ee2c47d9f26a5"));
        proof[1] = bytes32(abi.encodePacked(hex"662362a44a545cd42cdf7e9c1cfc7eb3b55ebb3af452e1bd516d7329f0f490fa"));
        proof[2] = bytes32(abi.encodePacked(hex"8a12e22ffa7163c5249a71f3df41704c2e2ccf8bab02dec02fc8db2740f51256"));
        proof[3] = bytes32(abi.encodePacked(hex"afb5ee202bbe624a5d933b1eda40f5bf6bcd6674dbf1af8eea698ae023c104fe"));

        bytes32 root = bytes32(abi.encodePacked(hex"1c8dd9ca252d7eb9bf1cccb9ab587e9a1dccca4c7474bb8739c0e5218964a2b4"));

        // expect revert because the total number of leaves is not correct
        vm.expectRevert(MerkleProof.InvalidProofLength.selector);
        MerkleProof.verify(proof, root, leaf, 7, 8); // totalLeaves should be 9, but we pass 8
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function test_revert_processProof_wrongProofLength() public {
        bytes32 leaf = keccak256(
            abi.encodePacked(keccak256(abi.encodePacked("bbn1j8k7l6m5n4o3p2q1r0s9t8u7v6w5x4y3z2a1b600000000000000000")))
        );

        bytes32[] memory proof = new bytes32[](3);
        proof[0] = bytes32(abi.encodePacked(hex"c99659b6e1c7a2df5c6ce352241ef43152f1fed170d2fdc8d67ee2c47d9f26a5"));
        proof[1] = bytes32(abi.encodePacked(hex"662362a44a545cd42cdf7e9c1cfc7eb3b55ebb3af452e1bd516d7329f0f490fa"));
        proof[2] = bytes32(abi.encodePacked(hex"8a12e22ffa7163c5249a71f3df41704c2e2ccf8bab02dec02fc8db2740f51256"));

        bytes32 root = bytes32(abi.encodePacked(hex"1c8dd9ca252d7eb9bf1cccb9ab587e9a1dccca4c7474bb8739c0e5218964a2b4"));

        // expect revert because the proof length must be 4
        vm.expectRevert(MerkleProof.InvalidProofLength.selector);
        MerkleProof.verify(proof, root, leaf, 7, 9);
    }

    function test_verify_largeTree_powerOf2Leaves_firstLeaf() public pure {
        bytes32 leaf = keccak256(
            abi.encodePacked(
                keccak256(abi.encodePacked("cosmwasm1su3v5d76jn20lprhtqp8zp7lltru62heu9pfnm2urata3k6a9m3srmm70p88953"))
            )
        );

        bytes32[] memory proof = new bytes32[](16);
        proof[0] = bytes32(abi.encodePacked(hex"673148d63ef55d0d7013b98e1183f906f58b1e7e7b6254efe73dd80cf059620d"));
        proof[1] = bytes32(abi.encodePacked(hex"1e4ab791007c87080775f67e42b1e5bd42cdf3dc372c82aa2a9088d2a8d83051"));
        proof[2] = bytes32(abi.encodePacked(hex"3af88ef901ccfca25edc7726e56663216281518a5e4ef731296ade06823490d0"));
        proof[3] = bytes32(abi.encodePacked(hex"89973dafd138e3f97d7d78ae850df128e2358d0b44bda1c0ae9d442b7263a29f"));
        proof[4] = bytes32(abi.encodePacked(hex"201f1474b251d8c59928594d5ab942437bea0ad957118f4173b341d73a585be9"));
        proof[5] = bytes32(abi.encodePacked(hex"177a97f75fcf0e54326fcb84d24373debc048dc5ce90798bc910f07bca0fdb46"));
        proof[6] = bytes32(abi.encodePacked(hex"5d86933ca5be44f28d16b10cffaabe539422d5d789a4579b3137c0c6e0ff1208"));
        proof[7] = bytes32(abi.encodePacked(hex"9a2426b8df26e9c71c3e6a8276dc29793255312a01e31f75a8bbec0034b070af"));
        proof[8] = bytes32(abi.encodePacked(hex"97d02a8b1382e9a4e53becd374f889418c9ea32ed112ac52e600a11e1276d3ab"));
        proof[9] = bytes32(abi.encodePacked(hex"a29134d83ced2e77e193fdf0202ccbc935fbfb881a357b21d0ad0adb5b53bf0c"));
        proof[10] = bytes32(abi.encodePacked(hex"776e99cc093db0e8da0ad3cee9065c80f4f865a0444b2d452a8c47422589fecf"));
        proof[11] = bytes32(abi.encodePacked(hex"4a890ee125015ec8ea962f3be27ccbc066bd19201a624010c65eb699999f0794"));
        proof[12] = bytes32(abi.encodePacked(hex"7bd61c04d8e8223609570e0d3fe76306e2f5de0625a5119698c9b304b3a42ae7"));
        proof[13] = bytes32(abi.encodePacked(hex"e5564a3f45a9fc226ef4d2b5e28bb77e44b8768d36b0cbf0aa1d52d29524b3d9"));
        proof[14] = bytes32(abi.encodePacked(hex"49403e80961cc67a99ecf5efa9bf09e05677f434f749bc6077fa23502a8a0300"));
        proof[15] = bytes32(abi.encodePacked(hex"922c23d52cb21fc4d74295dbcad1f138bcfb65c1a6c369ccd31aad6136f438ee"));

        bytes32 root = bytes32(abi.encodePacked(hex"af3e6afa64bf5592cf575a557f8d7b2762d9e8c1666b9f318d45eefc91602847"));

        assertTrue(MerkleProof.verify(proof, root, leaf, 0, 65536));
    }

    function test_verify_largeTree_powerOf2Leaves_middleLeaf() public pure {
        bytes32 leaf = keccak256(
            abi.encodePacked(
                keccak256(abi.encodePacked("cosmwasm1pu5myrun24yjrgpct27c2h4g87pd2yjhlj9ls3xv9n8t828xwxxsuus7js46050"))
            )
        );

        bytes32[] memory proof = new bytes32[](16);
        proof[0] = bytes32(abi.encodePacked(hex"72db77338bafd8acfe5e197772229b3e9ac73d5f0e842b2a1953ae7b144c789e"));
        proof[1] = bytes32(abi.encodePacked(hex"ce1f03042b510d1f4d1df11e3950f52107deef0d112078d960ebcc5d7ea9c71b"));
        proof[2] = bytes32(abi.encodePacked(hex"e41f55d2227cfdcfb62efcde61824b7c608a62e5c0956735b269a69e419f0e28"));
        proof[3] = bytes32(abi.encodePacked(hex"a297a36f424e9f4498ada0ccaa55db46222480038c30db3e7eafd5115eedda5d"));
        proof[4] = bytes32(abi.encodePacked(hex"5abb11e91bd8149df1175c003938791ecf76c12465ec1f492d5d8dfbfbac14b3"));
        proof[5] = bytes32(abi.encodePacked(hex"0e74fb2e7e37fc6f678d87f16e102bedef8dcd101871fa7516471afbe076bb4f"));
        proof[6] = bytes32(abi.encodePacked(hex"acfc0f2ea57d3f8ac2066082a42652c4059143712f98c7483547372e55ebd626"));
        proof[7] = bytes32(abi.encodePacked(hex"b90484e7c243904fe5aff46a710d7c4c12e54fa8536b14ad8b93b57243aa4281"));
        proof[8] = bytes32(abi.encodePacked(hex"e31e734a154275ae970295d5a1fc72db4776df7045ee1cdfba40405e8020f155"));
        proof[9] = bytes32(abi.encodePacked(hex"19cb2ce1ed2d7034ef4662ba6796eec4f29599acff59cdd62e3a6422db16cf40"));
        proof[10] = bytes32(abi.encodePacked(hex"699ee74997c17ab9dee38c8e05c492b4d74d97930b29e759323884384a56e15c"));
        proof[11] = bytes32(abi.encodePacked(hex"67f15101e1539f665ccf2534c8a3106a680b79b0b303f19de7e8029dfdece0bb"));
        proof[12] = bytes32(abi.encodePacked(hex"b06a8b31ed8562a634cb60d83f3cd2ce9cad2406bbab63546318733dea7d0a62"));
        proof[13] = bytes32(abi.encodePacked(hex"fd2add3e50b9c272aa670ac29824cda2a061867cd4885be160ce46609e88f4da"));
        proof[14] = bytes32(abi.encodePacked(hex"91b80ddde4c05e8123ad29bed131e965a39d70c305bc665dbda46b733f83097a"));
        proof[15] = bytes32(abi.encodePacked(hex"922c23d52cb21fc4d74295dbcad1f138bcfb65c1a6c369ccd31aad6136f438ee"));

        bytes32 root = bytes32(abi.encodePacked(hex"af3e6afa64bf5592cf575a557f8d7b2762d9e8c1666b9f318d45eefc91602847"));

        assertTrue(MerkleProof.verify(proof, root, leaf, 32767, 65536));
    }

    function test_verify_largeTree_powerOf2Leaves_lastLeaf() public pure {
        bytes32 leaf = keccak256(
            abi.encodePacked(
                keccak256(abi.encodePacked("cosmwasm1ydk8p72jmaz44v2k9psnv8h2ed5pgzzrkmkp8u4hpx6p9mywhh5s4w0fav42142"))
            )
        );

        bytes32[] memory proof = new bytes32[](16);
        proof[0] = bytes32(abi.encodePacked(hex"f9ea132a18d6f03d184abf640fcd11686c2b401e92aba0fd0fc7fd135abbc0c7"));
        proof[1] = bytes32(abi.encodePacked(hex"814d9fe42983b61bc9ab99b056a0ac4cbe87a1cf3030d7a8f7492f56aa590a9f"));
        proof[2] = bytes32(abi.encodePacked(hex"a78699979bd8aaafc0f160b6fa676c1a58fa8a6787eece2f18e6a85ced36b2f9"));
        proof[3] = bytes32(abi.encodePacked(hex"4d80b548beb73bd379e312d618ecd179c5770a5ac0f1b1eb67bbff9ea36c64aa"));
        proof[4] = bytes32(abi.encodePacked(hex"be9ae42169e7b91819204f207421ff677d8e30e88385b1a570c30a8fd7f7e931"));
        proof[5] = bytes32(abi.encodePacked(hex"568344d59f1b10770d931e2274fc8869255cabc34c06f18300f3fee3eb8d996d"));
        proof[6] = bytes32(abi.encodePacked(hex"c6d5d328680d72e7a78a44064a758274010c1b044dcd1ba0d7c81ef27f7a401c"));
        proof[7] = bytes32(abi.encodePacked(hex"f702b21ed362629b406365193ac3ade5f3487dd21fc8b753cbca619be3dfcddf"));
        proof[8] = bytes32(abi.encodePacked(hex"9662971499ee51876f9cfd910c3a02a266645d74ab1eb433367152f5f01807c7"));
        proof[9] = bytes32(abi.encodePacked(hex"695b4d5e4307922622aed867b4eaf7c45973493fbdd149e06481aeb0facb7e32"));
        proof[10] = bytes32(abi.encodePacked(hex"5a632dcf4efab387d6008508f207c7c6f72384b961635022187735194e2b579e"));
        proof[11] = bytes32(abi.encodePacked(hex"47453ebe7190d27d617fd2904e6b64e3582a8055d1e7c4611249d13dc534e1f1"));
        proof[12] = bytes32(abi.encodePacked(hex"de9803f48ec1372294b68a3cdcb9e05d10c0e0c2d43c49fb031bbe931593af7a"));
        proof[13] = bytes32(abi.encodePacked(hex"eb2d62f7fa962f420b56af2d4464b373d37c517c81337008e43cd4c63ec049f6"));
        proof[14] = bytes32(abi.encodePacked(hex"18b48888c20d4e568357b56653595c8035e406b26583e47a0e259cc1af2b42dc"));
        proof[15] = bytes32(abi.encodePacked(hex"ac7c966b9149642f7a0dc40964019052cf46d0cd6475db5c6b7791abf108661f"));

        bytes32 root = bytes32(abi.encodePacked(hex"af3e6afa64bf5592cf575a557f8d7b2762d9e8c1666b9f318d45eefc91602847"));

        assertTrue(MerkleProof.verify(proof, root, leaf, 65535, 65536));
    }

    function test_verify_largeTree_notPowerOf2Leaves_firstLeaf() public pure {
        bytes32 leaf = keccak256(
            abi.encodePacked(
                keccak256(abi.encodePacked("cosmwasm186cjfw3seu04n70c308hl6wr25cpg4pre6lyjhrurshm03yukz0qu6yllh43596"))
            )
        );

        bytes32[] memory proof = new bytes32[](17);
        proof[0] = bytes32(abi.encodePacked(hex"06d47c1cccfefefc4160583a0179d9b36f9f50b4820fc9c651ca9c409ceba9bc"));
        proof[1] = bytes32(abi.encodePacked(hex"b75002e96950f8629ab806918f61a116365489eee38d545543e07cfce8fee1dd"));
        proof[2] = bytes32(abi.encodePacked(hex"914b6df71717cc6b4a3c32da4bb78325d7165c27da8f96e86cdeb1369f2befa4"));
        proof[3] = bytes32(abi.encodePacked(hex"5997bd74ba0a8acccaab1fc04c53e2c43d6dc7ef3719866b8149cb93b1cc7460"));
        proof[4] = bytes32(abi.encodePacked(hex"12d189e67a266fc3121a0adbf350691f189afafbe497a857074692049e8ce7d2"));
        proof[5] = bytes32(abi.encodePacked(hex"8a0c3c62754b9ebbf3aa61f4dacb633d44e5fcc033120e71d5c6c024cb049cd2"));
        proof[6] = bytes32(abi.encodePacked(hex"05ed1a2871283daa6ba4423c8572da23b18ad47ec6c4f34a310b0b6a650f811c"));
        proof[7] = bytes32(abi.encodePacked(hex"38578f10bf20ff53aff0a0160c32556ac4b6db5564b642b70e31d27279dadf96"));
        proof[8] = bytes32(abi.encodePacked(hex"5bbf29a7c57e2dc405766936f1b0380407ec3605aec2dca739d35f74a4c1c779"));
        proof[9] = bytes32(abi.encodePacked(hex"a3819ae16d34b95cce57f6b560b60119da654c4917cc75fc117666f3c647f4a5"));
        proof[10] = bytes32(abi.encodePacked(hex"53368c9341d43c3149a550fc3cead19c3acecfcc93bb1834d24dd1339bc0a13c"));
        proof[11] = bytes32(abi.encodePacked(hex"9c773ea9c3e3f3721cf035cdde5dd139299ce1d4bb2125fedc3ab583371de40e"));
        proof[12] = bytes32(abi.encodePacked(hex"33b5e6814b4bdc75d73b6d55c5094dfb95dced5f10dc8cc83a006517add10b3c"));
        proof[13] = bytes32(abi.encodePacked(hex"9ae77f6473f43e9f44da6d235cc9f12908dd2a442eb1fae96a176e93a415671d"));
        proof[14] = bytes32(abi.encodePacked(hex"ed2f2b05aee5954bd1c1536e39e6ef1dfa709f526bbcc5e1c89b5e9da6a2caf4"));
        proof[15] = bytes32(abi.encodePacked(hex"64016884cdbedea77ac64b35a3589d3e6817d7017d3a8f8c30a85f0997a39895"));
        proof[16] = bytes32(abi.encodePacked(hex"64c18ec3dfea8d9b8294694df6a0155a9bf1cdd708987f38a2cc31687c9afb91"));

        bytes32 root = bytes32(abi.encodePacked(hex"7b4d904af34a97c66625d6d80c8c63a26ec3a48191840f325cd51d9218699466"));

        assertTrue(MerkleProof.verify(proof, root, leaf, 0, 100_000));
    }

    function test_verify_largeTree_notPowerOf2Leaves_middleLeaf() public pure {
        bytes32 leaf = keccak256(
            abi.encodePacked(
                keccak256(abi.encodePacked("cosmwasm1akatvcfps5k07fmpn53047cdl36dgxh98m2r9lxtl0z9nzyr3t3s0vhvh242322"))
            )
        );

        bytes32[] memory proof = new bytes32[](17);
        proof[0] = bytes32(abi.encodePacked(hex"0f527483eb604c672836418bd7fcf40f4f4a605cd31d2bb2def928031b1dc906"));
        proof[1] = bytes32(abi.encodePacked(hex"1d839df03bf226f0517ddc7187285beb33cfa491a1efe1c2fc2704e25d3b4405"));
        proof[2] = bytes32(abi.encodePacked(hex"b00d1deaf9d271a72338cdcb36cc4d813ccaa3230089a236788d3a844893742c"));
        proof[3] = bytes32(abi.encodePacked(hex"62e2ac64ea4341230aa60d98ad1347e705781e9834a3317b59a6556f9125a5d1"));
        proof[4] = bytes32(abi.encodePacked(hex"eb2e87dda6822ce7ef7487108c6b25661a29ef328949232f7eb55a90a625f7e6"));
        proof[5] = bytes32(abi.encodePacked(hex"68666fa2cb599e3a33444b57d392c5ee093b4ffc07aaaf05c05eb36591698943"));
        proof[6] = bytes32(abi.encodePacked(hex"0e6837f5c5506bc55562b997ff9ab7b881b2b02a1412dde86395fba4c9b67cc9"));
        proof[7] = bytes32(abi.encodePacked(hex"c1e7e1bd8cb182221052255a851ef8e27f76fd38186cb138d82374a8b68a9e14"));
        proof[8] = bytes32(abi.encodePacked(hex"c63b6aa2cd6cfb32ce97117ac04ebbee007a94c73dabf4fe423f029a49a50ab1"));
        proof[9] = bytes32(abi.encodePacked(hex"b2d250876ad1fc4c99e02818c4c4a42dbc306c00f8f6bfa3eeaed17e2c84554e"));
        proof[10] = bytes32(abi.encodePacked(hex"07ec433dc1ed6671e447ea837274b977527efae062be8e9f27454a1f9e1ecf74"));
        proof[11] = bytes32(abi.encodePacked(hex"efa23c7bc57681ebe4fa2b4acedf3c090dad69b509fa2d4a1b87f137fc5f2344"));
        proof[12] = bytes32(abi.encodePacked(hex"e4a2f68eb4ba01e96cbefd0101bd6d68dfb7e40768de702531100f8b09df59cb"));
        proof[13] = bytes32(abi.encodePacked(hex"d44bf5816a972ce6d0544029876ac335b3cf784b8bad9c14824e2e54a1a9d33e"));
        proof[14] = bytes32(abi.encodePacked(hex"05510077f3c8c5931ed80d4ec822510d260bb47cd20892aff4b5799049abfa2f"));
        proof[15] = bytes32(abi.encodePacked(hex"a70b33023da951a16f1970c014aba63aa1fe4cde3f90333a756096cdccfa3831"));
        proof[16] = bytes32(abi.encodePacked(hex"64c18ec3dfea8d9b8294694df6a0155a9bf1cdd708987f38a2cc31687c9afb91"));

        bytes32 root = bytes32(abi.encodePacked(hex"7b4d904af34a97c66625d6d80c8c63a26ec3a48191840f325cd51d9218699466"));

        assertTrue(MerkleProof.verify(proof, root, leaf, 49999, 100_000));
    }

    function test_verify_largeTree_notPowerOf2Leaves_lastLeaf() public pure {
        bytes32 leaf = keccak256(
            abi.encodePacked(
                keccak256(abi.encodePacked("cosmwasm1ky7599ft99fem2ghpawtk9sl6062wztzrzvrdevd7mrsw8dgdg8ssgktc023305"))
            )
        );

        bytes32[] memory proof = new bytes32[](17);
        proof[0] = bytes32(abi.encodePacked(hex"edc16948fd1c0ecb016c65d2158fef4e55ad47b456060540db927f62dadc1365"));
        proof[1] = bytes32(abi.encodePacked(hex"dbf7e3e384bd82e3ea4129465760b522602b4369a2ded4d07ca1918a617330ef"));
        proof[2] = bytes32(abi.encodePacked(hex"433fe4dbf16913540a84f2e34c0338ddeb1c5a82a4754ece36b73f119b3131db"));
        proof[3] = bytes32(abi.encodePacked(hex"bb480b01bee5f0a535faa8fd476573231ee01bc7be6cff4cf20ae179c49da0de"));
        proof[4] = bytes32(abi.encodePacked(hex"9f0fc858bef22cd4d20a7ed9ab0a7af657240dc4b5a543fcaf48c0472dbafd76"));
        proof[5] = bytes32(abi.encodePacked(hex"0eb01ebfc9ed27500cd4dfc979272d1f0913cc9f66540d7e8005811109e1cf2d"));
        proof[6] = bytes32(abi.encodePacked(hex"887c22bd8750d34016ac3c66b5ff102dacdd73f6b014e710b51e8022af9a1968"));
        proof[7] = bytes32(abi.encodePacked(hex"0571ffa8dc5b4add234a029bcf307f9ce50d4c7dd5cbafd03c4012e0aded8f2b"));
        proof[8] = bytes32(abi.encodePacked(hex"9867cc5f7f196b93bae1e27e6320742445d290f2263827498b54fec539f756af"));
        proof[9] = bytes32(abi.encodePacked(hex"1f9cb56f05fa512f3e3d64ae0b32cae39cd92a70a36dc69b3a97107246f20356"));
        proof[10] = bytes32(abi.encodePacked(hex"f5b265afc2e154c3b6e66dfd6429edf1cfff7773af85e5eeac7115534867184b"));
        proof[11] = bytes32(abi.encodePacked(hex"f8b13a49e282f609c317a833fb8d976d11517c571d1221a265d25af778ecf892"));
        proof[12] = bytes32(abi.encodePacked(hex"3490c6ceeb450aecdc82e28293031d10c7d73bf85e57bf041a97360aa2c5d99c"));
        proof[13] = bytes32(abi.encodePacked(hex"c1df82d9c4b87413eae2ef048f94b4d3554cea73d92b0f7af96e0271c691e2bb"));
        proof[14] = bytes32(abi.encodePacked(hex"5c67add7c6caf302256adedf7ab114da0acfe870d449a3a489f781d659e8becc"));
        proof[15] = bytes32(abi.encodePacked(hex"007307ad90731d061832d94d3a2336776a67b3f251ea1059a5e14c8d5b77ef5b"));
        proof[16] = bytes32(abi.encodePacked(hex"d32115598d70b0d31098600cd8e64f271a41fc14b66395f6304b156de27bea9e"));

        bytes32 root = bytes32(abi.encodePacked(hex"7b4d904af34a97c66625d6d80c8c63a26ec3a48191840f325cd51d9218699466"));

        assertTrue(MerkleProof.verify(proof, root, leaf, 99999, 100_000));
    }

    function test_revert_verify_incorrectProofForLeaf() public pure {
        bytes32 leaf = keccak256(
            abi.encodePacked(
                keccak256(abi.encodePacked("cosmwasm1ky7599ft99fem2ghpawtk9sl6062wztzrzvrdevd7mrsw8dgdg8ssgktc023305"))
            )
        );

        bytes32[] memory proof = new bytes32[](17);
        proof[0] = bytes32(abi.encodePacked(hex"adb16948fd1c0ecb016c65d2158fef4e55ad47b456060540db927f62dadc1365"));
        proof[1] = bytes32(abi.encodePacked(hex"abb7e3e384bd82e3ea4129465760b522602b4369a2ded4d07ca1918a617330ef"));
        proof[2] = bytes32(abi.encodePacked(hex"a3bfe4dbf16913540a84f2e34c0338ddeb1c5a82a4754ece36b73f119b3131db"));
        proof[3] = bytes32(abi.encodePacked(hex"abb80b01bee5f0a535faa8fd476573231ee01bc7be6cff4cf20ae179c49da0de"));
        proof[4] = bytes32(abi.encodePacked(hex"afbfc858bef22cd4d20a7ed9ab0a7af657240dc4b5a543fcaf48c0472dbafd76"));
        proof[5] = bytes32(abi.encodePacked(hex"aeb01ebfc9ed27500cd4dfc979272d1f0913cc9f66540d7e8005811109e1cf2d"));
        proof[6] = bytes32(abi.encodePacked(hex"a8bc22bd8750d34016ac3c66b5ff102dacdd73f6b014e710b51e8022af9a1968"));
        proof[7] = bytes32(abi.encodePacked(hex"a5b1ffa8dc5b4add234a029bcf307f9ce50d4c7dd5cbafd03c4012e0aded8f2b"));
        proof[8] = bytes32(abi.encodePacked(hex"a8b7cc5f7f196b93bae1e27e6320742445d290f2263827498b54fec539f756bf"));
        proof[9] = bytes32(abi.encodePacked(hex"afbcb56f05fa512f3e3d64ae0b32cae39cd92a70a36dc69b3a97107246f203b6"));
        proof[10] = bytes32(abi.encodePacked(hex"a5b265afc2e154c3b6e66dfd6429edf1cfff7773af85e5eeac7115534867184b"));
        proof[11] = bytes32(abi.encodePacked(hex"a8b13a49e282f609c317a833fb8d976d11517c571d1221a265d25af778ecf892"));
        proof[12] = bytes32(abi.encodePacked(hex"a4b0c6ceeb450aecdc82e28293031d10c7d73bf85e57bf041a97360aa2c5d99c"));
        proof[13] = bytes32(abi.encodePacked(hex"a1bf82d9c4b87413eae2ef048f94b4d3554cea73d92b0f7af96e0271c691e2bb"));
        proof[14] = bytes32(abi.encodePacked(hex"acb7add7c6caf302256adedf7ab114da0acfe870d449a3a489f781d659e8becc"));
        proof[15] = bytes32(abi.encodePacked(hex"a0b307ad90731d061832d94d3a2336776a67b3f251ea1059a5e14c8d5b77ef5b"));
        proof[16] = bytes32(abi.encodePacked(hex"a3b115598d70b0d31098600cd8e64f271a41fc14b66395f6304b156de27bea9e"));

        bytes32 root = bytes32(abi.encodePacked(hex"7b4d904af34a97c66625d6d80c8c63a26ec3a48191840f325cd51d9218699466"));

        assertFalse(MerkleProof.verify(proof, root, leaf, 99999, 100_000));
    }

    function test_revert_verify_nonExistentLeaf() public pure {
        bytes32 leaf = keccak256(abi.encodePacked(keccak256(abi.encodePacked("This leaf is not a member of the tree"))));

        bytes32[] memory proof = new bytes32[](17);
        proof[0] = bytes32(abi.encodePacked(hex"edc16948fd1c0ecb016c65d2158fef4e55ad47b456060540db927f62dadc1365"));
        proof[1] = bytes32(abi.encodePacked(hex"dbf7e3e384bd82e3ea4129465760b522602b4369a2ded4d07ca1918a617330ef"));
        proof[2] = bytes32(abi.encodePacked(hex"433fe4dbf16913540a84f2e34c0338ddeb1c5a82a4754ece36b73f119b3131db"));
        proof[3] = bytes32(abi.encodePacked(hex"bb480b01bee5f0a535faa8fd476573231ee01bc7be6cff4cf20ae179c49da0de"));
        proof[4] = bytes32(abi.encodePacked(hex"9f0fc858bef22cd4d20a7ed9ab0a7af657240dc4b5a543fcaf48c0472dbafd76"));
        proof[5] = bytes32(abi.encodePacked(hex"0eb01ebfc9ed27500cd4dfc979272d1f0913cc9f66540d7e8005811109e1cf2d"));
        proof[6] = bytes32(abi.encodePacked(hex"887c22bd8750d34016ac3c66b5ff102dacdd73f6b014e710b51e8022af9a1968"));
        proof[7] = bytes32(abi.encodePacked(hex"0571ffa8dc5b4add234a029bcf307f9ce50d4c7dd5cbafd03c4012e0aded8f2b"));
        proof[8] = bytes32(abi.encodePacked(hex"9867cc5f7f196b93bae1e27e6320742445d290f2263827498b54fec539f756af"));
        proof[9] = bytes32(abi.encodePacked(hex"1f9cb56f05fa512f3e3d64ae0b32cae39cd92a70a36dc69b3a97107246f20356"));
        proof[10] = bytes32(abi.encodePacked(hex"f5b265afc2e154c3b6e66dfd6429edf1cfff7773af85e5eeac7115534867184b"));
        proof[11] = bytes32(abi.encodePacked(hex"f8b13a49e282f609c317a833fb8d976d11517c571d1221a265d25af778ecf892"));
        proof[12] = bytes32(abi.encodePacked(hex"3490c6ceeb450aecdc82e28293031d10c7d73bf85e57bf041a97360aa2c5d99c"));
        proof[13] = bytes32(abi.encodePacked(hex"c1df82d9c4b87413eae2ef048f94b4d3554cea73d92b0f7af96e0271c691e2bb"));
        proof[14] = bytes32(abi.encodePacked(hex"5c67add7c6caf302256adedf7ab114da0acfe870d449a3a489f781d659e8becc"));
        proof[15] = bytes32(abi.encodePacked(hex"007307ad90731d061832d94d3a2336776a67b3f251ea1059a5e14c8d5b77ef5b"));
        proof[16] = bytes32(abi.encodePacked(hex"d32115598d70b0d31098600cd8e64f271a41fc14b66395f6304b156de27bea9e"));

        bytes32 root = bytes32(abi.encodePacked(hex"7b4d904af34a97c66625d6d80c8c63a26ec3a48191840f325cd51d9218699466"));

        assertFalse(MerkleProof.verify(proof, root, leaf, 99999, 100_000));
    }
}
