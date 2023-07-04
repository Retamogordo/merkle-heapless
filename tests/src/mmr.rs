#[cfg(test)]
mod tests {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    use merkle_heapless::mmr_macro;
    use merkle_heapless::traits::{AppendOnly, HashT, ProofValidator, StaticTreeTrait};

    #[derive(Debug)]
    pub struct StdHash;

    impl HashT for StdHash {
        type Output = [u8; 8];

        fn hash(input: &[u8]) -> Self::Output {
            let mut s = DefaultHasher::new();
            input.hash(&mut s);
            s.finish().to_ne_bytes()
        }
    }

    #[derive(Debug)]
    pub struct Blake2_256Hash;

    impl HashT for Blake2_256Hash {
        type Output = [u8; 32];

        fn hash(input: &[u8]) -> Self::Output {
            sp_core::blake2_256(input)
        }
    }

    #[test]
    fn mmr_binary() {
        mmr_macro::mmr!(Type = FooMMR, BranchFactor = 2, Peaks = 7, Hash = StdHash);
        //        let mut mmr = FooMMR::from(FooMMRPeak::Peak0(Default::default())).unwrap();
        let mut mmr = FooMMR::default();
        // peak leaf numbers: [0, 0, 0, 0, 0]
        assert_eq!(mmr.peaks()[0].height(), 0);

        mmr.try_append(b"apple").unwrap();
        // peak leaf numbers: [1, 0, 0, 0, 0]
        assert_eq!(mmr.peaks()[0].height(), 0);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 1);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 0);
        let proof = mmr.generate_proof(0);
        let res = proof.validate(b"apple");
        assert!(res);

        mmr.try_append(b"banana").unwrap();
        assert_eq!(mmr.peaks()[0].height(), 1);
        // peak leaf numbers: [2, 0, 0, 0, 0] because 1, 1 is merged -> 2, 0
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 2);
        assert_eq!(mmr.peaks()[1].height(), 0);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 0);
        let proof = mmr.generate_proof(1);
        let res = proof.validate(b"banana");
        assert!(res);

        mmr.try_append(b"cherry").unwrap();
        // peak leaf numbers: [2, 1, 0, 0, 0]
        assert_eq!(mmr.peaks()[0].height(), 1);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 2);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 1);
        let proof = mmr.generate_proof(2);
        let res = proof.validate(b"cherry");
        assert!(res);

        mmr.try_append(b"kiwi").unwrap();
        // peak leaf numbers: [4, 0, 0, 0, 0] because 2, 1, 1 is merged -> 2, 2, 0 -> 4, 0, 0
        assert_eq!(mmr.peaks()[0].height(), 2);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 4);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 0);
        let proof = mmr.generate_proof(3);
        let res = proof.validate(b"kiwi");
        assert!(res);

        mmr.try_append(b"lemon").unwrap();
        // peak leaf numbers: [4, 1, 0, 0, 0]
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 4);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 1);
        let proof = mmr.generate_proof(4);
        let res = proof.validate(b"lemon");
        assert!(res);

        mmr.try_append(b"lime").unwrap();
        // peak leaf numbers: [4, 2, 0, 0, 0]
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 4);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 2);
        let proof = mmr.generate_proof(5);
        let res = proof.validate(b"lime");
        assert!(res);

        mmr.try_append(b"mango").unwrap();
        // peak leaf numbers: [4, 2, 1, 0, 0]
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 4);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 2);
        assert_eq!(mmr.peaks()[2].num_of_leaves(), 1);

        mmr.try_append(b"carrot").unwrap();
        // peak leaf numbers: [8, 0, 0, 0, 0]
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 8);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 0);

        mmr.try_append(b"peach").unwrap();
        // peak leaf numbers: [8, 1, 0, 0, 0]
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 8);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 1);

        mmr.try_append(b"pear").unwrap();
        // peak leaf numbers: [8, 2, 0, 0, 0]
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 8);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 2);

        mmr.try_append(b"potato").unwrap();
        // peak leaf numbers: [8, 2, 1, 0, 0]
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 8);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 2);
        assert_eq!(mmr.peaks()[2].num_of_leaves(), 1);

        mmr.try_append(b"strawberry").unwrap();
        // peak leaf numbers: [8, 4, 0, 0, 0]
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 8);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 4);
    }

    #[test]
    fn mmr_binary_1_peak() {
        mmr_macro::mmr!(BranchFactor = 2, Peaks = 1, Hash = StdHash);

        let mut mmr = MerkleMountainRange::default();

        mmr.try_append(b"apple").unwrap();
        // peak leaf numbers: [1]
        assert_eq!(mmr.peaks()[0].height(), 0);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 1);
        let proof = mmr.generate_proof(0);
        let res = proof.validate(b"apple");
        assert!(res);

        mmr.try_append(b"banana").unwrap();
        // peak leaf numbers: [1]
        assert_eq!(mmr.peaks()[0].height(), 1);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 2);
        let proof = mmr.generate_proof(1);
        let res = proof.validate(b"banana");
        assert!(res);

        assert!(mmr.try_append(b"cherry").is_err());
        assert_eq!(mmr.peaks()[0].height(), 1);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 2);
    }

    #[test]
    fn mmr_binary_2_peaks() {
        mmr_macro::mmr!(BranchFactor = 2, Peaks = 2, Hash = StdHash);

        let mut mmr = MerkleMountainRange::default();

        mmr.try_append(b"apple").unwrap();
        // peak leaf numbers: [1, 0]
        assert_eq!(mmr.peaks()[0].height(), 0);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 1);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 0);
        let proof = mmr.generate_proof(0);
        let res = proof.validate(b"apple");
        assert!(res);

        mmr.try_append(b"ananas").unwrap();
        // peak leaf numbers: [2, 0]
        assert_eq!(mmr.peaks()[0].height(), 1);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 2);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 0);
        let proof = mmr.generate_proof(1);
        let res = proof.validate(b"ananas");
        assert!(res);

        mmr.try_append(b"banana").unwrap();
        // peak leaf numbers: [2, 1]
        assert_eq!(mmr.peaks()[0].height(), 1);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 2);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 1);
        let proof = mmr.generate_proof(2);
        let res = proof.validate(b"banana");
        assert!(res);

        mmr.try_append(b"berry").unwrap();
        // peak leaf numbers: [4, 0]
        assert_eq!(mmr.peaks()[0].height(), 2);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 4);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 0);
        let proof = mmr.generate_proof(3);
        let res = proof.validate(b"berry");
        assert!(res);

        mmr.try_append(b"cherry").unwrap();
        // peak leaf numbers: [4, 1]
        assert_eq!(mmr.peaks()[0].height(), 2);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 4);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 1);
        let proof = mmr.generate_proof(4);
        let res = proof.validate(b"cherry");
        assert!(res);

        mmr.try_append(b"kiwi").unwrap();
        // peak leaf numbers: [4, 2]
        assert_eq!(mmr.peaks()[0].height(), 2);
        assert_eq!(mmr.peaks()[1].height(), 1);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 4);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 2);
        let proof = mmr.generate_proof(5);
        let res = proof.validate(b"kiwi");
        assert!(res);

        mmr.try_append(b"lemon").unwrap();
        // peak leaf numbers: [4, 3]
        assert_eq!(mmr.peaks()[0].height(), 2);
        assert_eq!(mmr.peaks()[1].height(), 2);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 4);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 3);
        let proof = mmr.generate_proof(6);
        let res = proof.validate(b"lemon");
        assert!(res);

        mmr.try_append(b"lime").unwrap();
        // peak leaf numbers: [4, 3]
        assert_eq!(mmr.peaks()[0].height(), 2);
        assert_eq!(mmr.peaks()[1].height(), 2);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 4);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 4);
        let proof = mmr.generate_proof(7);
        let res = proof.validate(b"lime");
        assert!(res);

        assert!(mmr.try_append(b"mango").is_err());
    }

    #[test]
    fn create_from_peak() {
        mmr_macro::mmr!(BranchFactor = 2, Peaks = 5, Hash = StdHash);

        let mut mmr =
            MerkleMountainRange::from_peak(MerkleMountainRangePeak::Peak0(Default::default()));
        assert_eq!(mmr.peaks()[0].height(), 5 - 0);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 0);
        for i in 0u8..32 {
            assert_eq!(mmr.peaks()[0].height(), 5 - 0);
            assert_eq!(mmr.peaks()[0].num_of_leaves(), i as usize);
            assert!(mmr.try_append(&[i]).is_ok());
        }
        assert!(mmr.try_append(b"apple").is_ok());
        assert_eq!(mmr.peaks()[0].height(), 5 - 0);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 32);
        assert_eq!(mmr.peaks()[1].height(), 0);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 1);
        let proof = mmr.generate_proof(32);
        let res = proof.validate(b"apple");
        assert!(res);

        let mmr =
            MerkleMountainRange::from_peak(MerkleMountainRangePeak::Peak1(Default::default()));
        assert_eq!(mmr.peaks()[0].height(), 5 - 1);
        let mmr =
            MerkleMountainRange::from_peak(MerkleMountainRangePeak::Peak2(Default::default()));
        assert_eq!(mmr.peaks()[0].height(), 5 - 2);
        let mmr =
            MerkleMountainRange::from_peak(MerkleMountainRangePeak::Peak3(Default::default()));
        assert_eq!(mmr.peaks()[0].height(), 5 - 3);
        let mmr =
            MerkleMountainRange::from_peak(MerkleMountainRangePeak::Peak4(Default::default()));
        assert_eq!(mmr.peaks()[0].height(), 5 - 4);
        let mmr =
            MerkleMountainRange::from_peak(MerkleMountainRangePeak::Peak5(Default::default()));
        assert_eq!(mmr.peaks()[0].height(), 5 - 5);
    }

    #[test]
    fn mmr_binary_4_peaks() {
        //        use crate::Blake2_256Hash;

        mmr_macro::mmr!(BranchFactor = 4, Peaks = 3, Hash = Blake2_256Hash);

        let mut mmr = MerkleMountainRange::default();
        assert_eq!(mmr.base_layer_size(), 3);
        assert_eq!(mmr.peaks()[0].height(), 0);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 0);
        // fill the first peak
        for i in 0u8..64 {
            assert!(mmr.try_append(&[i]).is_ok());
            let proof = mmr.generate_proof(i as usize);
            let res = proof.validate(&[i]);
            assert!(res);
        }
        let mut it = mmr.peaks().iter();
        assert_eq!(it.next().unwrap().height(), 3);
        for peak in it {
            assert_eq!(peak.height(), 0);
        }

        // fill the second peak
        for i in 64u8..128 {
            assert!(mmr.try_append(&[i]).is_ok());
            let proof = mmr.generate_proof(i as usize);
            let res = proof.validate(&[i]);
            assert!(res);
        }
        let mut it = mmr.peaks().iter();
        assert_eq!(it.next().unwrap().height(), 3);
        assert_eq!(it.next().unwrap().height(), 3);
        for peak in it {
            assert_eq!(peak.height(), 0);
        }

        // fill the third peak
        for i in 128u8..192 {
            assert!(mmr.try_append(&[i]).is_ok());
            let proof = mmr.generate_proof(i as usize);
            let res = proof.validate(&[i]);
            assert!(res);
        }
        for peak in mmr.peaks().iter() {
            assert_eq!(peak.height(), 3);
        }
        // no more space in mmr
        assert!(mmr.try_append(&[192]).is_err())
    }
}
