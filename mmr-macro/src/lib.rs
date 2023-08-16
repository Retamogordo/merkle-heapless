//! # Merkle Mountain Range macro
//! Include ["mmr_macro"] feature in merkle-heapless dependency
//! ### Necessary compiler features
//! ```rust
//! // compulsory at the beginning of the .rs file in order the macro to compile
//! #![allow(incomplete_features)]
//! #![feature(generic_const_exprs)]
//! // snip
//! ### Declaration and instantiation
//! use merkle_heapless::{mmr_macro};
//! // declaration with expicit type name for your MMR
//! mmr_macro::mmr!(Type = FooMMR, BranchFactor = 2, Peaks = 3, Hash = StdHash, MaxInputWordLength = 10);
//! let mmr = FooMMR::default();
//! // implicitly creates MerkleMountainRange type
//! mmr_macro::mmr!(BranchFactor = 2, Peaks = 5, Hash = StdHash, MaxInputWordLength = 10);
//! // create with default current peak of height 0
//! let mmr = MerkleMountainRange::default();
//! // or create with current peak of height 2
//! let mmr = MerkleMountainRange::from_peak(MerkleMountainRangePeak::Peak3(Default::default()));
//! assert_eq!(mmr.peaks()[0].height(), 5 - 3);
//! ```
//! ### Functionality
//! The functionality of Mountain Range is similar to that of the Merkle tree.   
//! ```rust
//! mmr.try_append(b"apple").unwrap();
//! // peak leaf numbers: [1, 0, 0, 0, 0]
//! assert_eq!(mmr.peaks()[0].height(), 0);
//! assert_eq!(mmr.peaks()[0].num_of_leaves(), 1);
//! assert_eq!(mmr.peaks()[1].num_of_leaves(), 0);
//! let proof = mmr.generate_proof(0);
//! assert!(proof.validate(b"apple"));
//! ```

use convert_case::{Case, Casing};
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{Error, Ident, LitInt, Token};

struct MMRInput {
    mmr_type: String,
    num_of_peaks: usize,
    branch_factor: usize,
    hash_type: String,
    max_input_len: usize,
}

impl MMRInput {
    const TYPE_IDENT: &str = "Type";
    const BRANCH_FACTOR_IDENT: &str = "BranchFactor";
    const NUM_OF_PEAKS: &str = "Peaks";
    const HASH_TYPE_IDENT: &str = "Hash";
    const MAX_INPUT_LEN_IDENT: &str = "MaxInputWordLength";
}

impl Parse for MMRInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut with_type = false;
        let maybe_type_ident = input.parse::<Ident>()?;
        let mmr_type: String;

        if maybe_type_ident == Self::TYPE_IDENT {
            with_type = true;
            input.parse::<Token![=]>()?;
            mmr_type = input.parse::<Ident>()?.to_string();
            input.parse::<Token![,]>()?;
        } else {
            mmr_type = "MerkleMountainRange".to_owned();
        }

        let err_msg = "error while parsing 'BranchFactor = <power of 2 number>' section";
        let branch_factor_ident = if with_type {
            input.parse::<Ident>().expect(err_msg)
        } else {
            maybe_type_ident
        };

        if branch_factor_ident != Self::BRANCH_FACTOR_IDENT {
            return Err(Error::new(
                branch_factor_ident.span(),
                format!("expected {}", Self::BRANCH_FACTOR_IDENT),
            ));
        }
        input.parse::<Token![=]>()?;
        let branch_factor: LitInt = input.parse().expect(err_msg);

        let err_msg = "error while parsing 'Peaks = <peak number>' section";
        input.parse::<Token![,]>().expect(err_msg);

        let num_of_peaks_ident = input.parse::<Ident>().expect(err_msg);
        if num_of_peaks_ident != Self::NUM_OF_PEAKS {
            return Err(Error::new(
                num_of_peaks_ident.span(),
                format!("expected {}", Self::NUM_OF_PEAKS),
            ));
        }
        input.parse::<Token![=]>().expect(err_msg);
        let num_of_peaks: LitInt = input.parse().expect(err_msg);

        let err_msg = "error while parsing 'Hash = <hash impl>' section";
        input.parse::<Token![,]>().expect(err_msg);

        let hash_type_ident = input.parse::<Ident>().expect(err_msg);
        if hash_type_ident != Self::HASH_TYPE_IDENT {
            return Err(Error::new(
                hash_type_ident.span(),
                format!("{}, expected {}", err_msg, Self::HASH_TYPE_IDENT),
            ));
        }
        input.parse::<Token![=]>().expect(err_msg);
        let hash_type = input.parse::<Ident>().expect(err_msg).to_string();

        let err_msg = "error while parsing 'MaxInputWordLength = <usize>' section";
        input.parse::<Token![,]>().expect(err_msg);

        let max_input_len_ident = input.parse::<Ident>().expect(err_msg);
        if max_input_len_ident != Self::MAX_INPUT_LEN_IDENT {
            return Err(Error::new(
                max_input_len_ident.span(),
                format!("{}, expected {}", err_msg, Self::MAX_INPUT_LEN_IDENT),
            ));
        }
        input.parse::<Token![=]>().expect(err_msg);
        let max_input_len: LitInt = input.parse().expect(err_msg);
        let max_input_len = max_input_len.base10_parse::<usize>()?;

        Ok(Self {
            mmr_type,
            num_of_peaks: num_of_peaks.base10_parse::<usize>()?,
            branch_factor: branch_factor.base10_parse::<usize>()?,
            hash_type,
            max_input_len,
        })
    }
}

#[proc_macro]
pub fn mmr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as MMRInput);

    if input.num_of_peaks < 2 {
        panic!("Number of peaks must be greater than 1");
    }

    let peak_height = input.num_of_peaks;
    let summit_height = (8 * core::mem::size_of::<usize>() as u32
        - input.num_of_peaks.leading_zeros()) as usize
        + 1;
    let total_height = summit_height + peak_height;

    let total_height = LitInt::new(&total_height.to_string(), proc_macro2::Span::call_site());
    let summit_height = LitInt::new(&summit_height.to_string(), proc_macro2::Span::call_site());
    let num_of_peaks = LitInt::new(
        &input.num_of_peaks.to_string(),
        proc_macro2::Span::call_site(),
    );
    let mmr_type = syn::Ident::new(&input.mmr_type, proc_macro2::Span::call_site());
    let mmr_peak_type = syn::Ident::new(
        &format!("{}Peak", input.mmr_type),
        proc_macro2::Span::call_site(),
    );
    let mmr_peak_proof_type = syn::Ident::new(
        &format!("{}PeakProof", input.mmr_type),
        proc_macro2::Span::call_site(),
    );
    let mmr_proof_type = syn::Ident::new(
        &format!("{}MMRProof", input.mmr_type),
        proc_macro2::Span::call_site(),
    );
    let hash_type = syn::Ident::new(&input.hash_type, proc_macro2::Span::call_site());
    let max_input_len = LitInt::new(
        &input.max_input_len.to_string(),
        proc_macro2::Span::call_site(),
    );

    let mod_ident = syn::Ident::new(
        &input.mmr_type.to_case(Case::Snake),
        proc_macro2::Span::call_site(),
    );

    let peak_variant_def_idents = (0..input.num_of_peaks)
        .map(|i| {
            (
                syn::Ident::new(&format!("Peak{i}"), proc_macro2::Span::call_site()),
                LitInt::new(
                    &(input.num_of_peaks - i).to_string(),
                    proc_macro2::Span::call_site(),
                ),
            )
        })
        .collect::<Vec<(syn::Ident, LitInt)>>();

    let branch_factor = LitInt::new(
        &input.branch_factor.to_string(),
        proc_macro2::Span::call_site(),
    );

    let peak_variant_def_tokens = peak_variant_def_idents.iter()
        .map(|(peak_lit, peak_height)| {

            quote! {
                #peak_lit(AugmentableTree<#branch_factor, #peak_height, #hash_type, #max_input_len, #mmr_peak_proof_type>)
            }
        })
        .collect::<Vec<_>>();

    let clone_impl_variant_def_tokens = peak_variant_def_idents
        .iter()
        .map(|(peak_lit, _)| {
            quote! {
                #peak_lit(tree) => #peak_lit(tree.clone())
            }
        })
        .collect::<Vec<_>>();

    let default_variant_def_token = peak_variant_def_idents
        .iter()
        .last()
        .map(|(peak_lit, _)| {
            quote! {
                #peak_lit(AugmentableTree::default())
            }
        })
        .expect("variant list is not empty. qed");

    let mut it1 = peak_variant_def_idents.iter().map(|(peak_lit, _)| peak_lit);
    let it2 = peak_variant_def_idents.iter().map(|(peak_lit, _)| peak_lit);
    it1.next();
    let augment_and_merge_variant_def_tokens = it1.zip(it2)
        .map(|(peak_lit1, peak_lit2)| {
            quote! {
                (#peak_lit1(this), #peak_lit1(other)) => Ok(#peak_lit2(this.augment_and_merge(other)))
            }
        })
        .collect::<Vec<_>>();

    let mut it1 = peak_variant_def_idents.iter().map(|(peak_lit, _)| peak_lit);
    let it2 = peak_variant_def_idents.iter().map(|(peak_lit, _)| peak_lit);
    it1.next();
    let augment_variant_def_tokens = it1
        .zip(it2)
        .map(|(peak_lit1, peak_lit2)| {
            quote! {
                #peak_lit1(this) => Ok(#peak_lit2(this.augment()))
            }
        })
        .collect::<Vec<_>>();

    let as_dyn_tree_variant_def_token = peak_variant_def_idents.iter()
        .map(|(peak_lit, _)| {
            quote! {
                #peak_lit(tree) => tree as &dyn merkle_heapless::traits::StaticTreeTrait<#branch_factor, #hash_type, #mmr_peak_proof_type>
            }
        })
        .collect::<Vec<_>>();

    let as_mut_dyn_tree_variant_def_token = peak_variant_def_idents.iter()
        .map(|(peak_lit, _)| {
            quote! {
                #peak_lit(tree) => tree as &mut dyn merkle_heapless::traits::StaticTreeTrait<#branch_factor, #hash_type, #mmr_peak_proof_type>
            }
        })
        .collect::<Vec<_>>();

    let as_append_only_variant_def_token = peak_variant_def_idents
        .iter()
        .map(|(peak_lit, _)| {
            quote! {
                #peak_lit(tree) => tree as &dyn merkle_heapless::traits::AppendOnly
            }
        })
        .collect::<Vec<_>>();

    let as_mut_append_only_variant_def_token = peak_variant_def_idents
        .iter()
        .map(|(peak_lit, _)| {
            quote! {
                #peak_lit(tree) => tree as &mut dyn merkle_heapless::traits::AppendOnly
            }
        })
        .collect::<Vec<_>>();

    let impl_method_body_token = quote! {
        use #mmr_peak_type::*;
        match self {
            #(#as_dyn_tree_variant_def_token),*
        }
    };
    let impl_mut_method_body_token = quote! {
        use #mmr_peak_type::*;
        match self {
            #(#as_mut_dyn_tree_variant_def_token),*
        }
    };

    let impl_append_only_method_body_token = quote! {
        use #mmr_peak_type::*;
        match self {
            #(#as_append_only_variant_def_token),*
        }
    };

    let impl_mut_append_only_method_body_token = quote! {
        use #mmr_peak_type::*;
        match self {
            #(#as_mut_append_only_variant_def_token),*
        }
    };

    let output = quote! {
            mod #mod_ident {
                use merkle_heapless::{StaticTree, Error};
                use merkle_heapless::augmentable::{AugmentableTree};
                use merkle_heapless::traits::{HashT, StaticTreeTrait, AppendOnly};
                use merkle_heapless::proof::{Proof, chain_proofs};
                use merkle_heapless::prefixed::{Prefixed};
                use super::#hash_type;

                type #mmr_peak_proof_type = Proof<#branch_factor, #peak_height, #hash_type, #max_input_len>;
                type #mmr_proof_type = Proof<#branch_factor, #total_height, #hash_type, #max_input_len>;

                #[derive(Debug)]
                pub enum #mmr_peak_type {
                    #(#peak_variant_def_tokens),*
                }

                impl Clone for #mmr_peak_type {
                    fn clone(&self) -> Self {
                        use #mmr_peak_type::*;
                        match self {
                            #(#clone_impl_variant_def_tokens),*
                        }
                    }
                }

                impl Default for #mmr_peak_type {
                    fn default() -> Self {
                        use #mmr_peak_type::*;
                        #default_variant_def_token
                    }
                }

                impl Copy for #mmr_peak_type {}

                impl #mmr_peak_type {
                    pub fn try_augment_and_merge(self, other: Self) -> Result<Self, Error> {
                        use #mmr_peak_type::{*};
                        match (self, other) {
                            #(#augment_and_merge_variant_def_tokens),*,
                            _ => Err(Error::Merge),
                        }
                    }
                    pub fn try_augment(self) -> Result<Self, Error> {
                        use #mmr_peak_type::{*};
                        match self {
                            #(#augment_variant_def_tokens),*,
                            _ => Err(Error::Merge),
                        }
                    }
                }

                impl StaticTreeTrait<#branch_factor, #hash_type, #mmr_peak_proof_type> for #mmr_peak_type {
                    fn generate_proof(&mut self, index: usize) -> #mmr_peak_proof_type {
                        #impl_mut_method_body_token.generate_proof(index)
                    }
                    fn replace(&mut self, index: usize, input: &[u8]) {
                        #impl_mut_method_body_token.replace(index, input)
                    }
                    fn replace_leaf(&mut self, index: usize, leaf: <#hash_type as HashT>::Output) {
                        #impl_mut_method_body_token.replace_leaf(index, leaf)
                    }
                    fn root(&self) -> <#hash_type as HashT>::Output {
                        #impl_method_body_token.root()
                    }
                    fn leaves(&self) -> &[Prefixed<#branch_factor, #hash_type>] {
                        #impl_method_body_token.leaves()
                    }
                    fn base_layer_size(&self) -> usize {
                        #impl_method_body_token.base_layer_size()
                    }
                    fn branch_factor(&self) -> usize {
                        #impl_method_body_token.branch_factor()
                    }
                    fn height(&self) -> usize {
                        #impl_method_body_token.height()
                    }
                }

                impl AppendOnly for #mmr_peak_type {
                    fn try_append(&mut self, input: &[u8]) -> Result<(), Error> {
                        #impl_mut_append_only_method_body_token.try_append(input)
                    }
                    fn num_of_leaves(&self) -> usize {
                        #impl_append_only_method_body_token.num_of_leaves()
                    }
                }

                pub struct #mmr_type
                where
                    [(); #num_of_peaks]: Sized,
                {
                    // the tree that generates the entire proof by chaining a peak's proof
                    summit_tree: StaticTree<#branch_factor, #summit_height, #hash_type, #max_input_len>,
                    peaks: [#mmr_peak_type; #num_of_peaks],
                    curr_peak_index: usize,
                    num_of_leaves: usize,
                }

                impl #mmr_type
                where
                    [(); #num_of_peaks]: Sized,
                {
                    pub fn from_peak(peak: #mmr_peak_type) -> Self {
                        let mut this = Self {
                            summit_tree: StaticTree::<#branch_factor, #summit_height, #hash_type, #max_input_len>::default(),
                            peaks: [#mmr_peak_type::default(); #num_of_peaks],
                            curr_peak_index: 0,
                            num_of_leaves: 0,
                        };
                        this.peaks[0] = peak;
                        this
                    }

                    fn merge_collapse(&mut self) {
                        let mut i = self.curr_peak_index;
                        // back propagate and merge peaks while possible
                        // the indicator that two peaks can merge is that they have the same rank (can check height or num_of_leaves)
                        while i > 0
    //                        && self.peaks[i].height() == self.peaks[i - 1].height()
                            && self.peaks[i].num_of_leaves() == self.peaks[i - 1].num_of_leaves() {

                            match self.peaks[i - 1].try_augment_and_merge(self.peaks[i]) {
                                Ok(peak) => { self.peaks[i - 1] = peak; },
                                Err(_) => break,
                            }
                            self.peaks[i] = Default::default();
                            i -= 1;
                        }
                        self.curr_peak_index = i;
                    }

                    pub fn try_append(&mut self, input: &[u8]) -> Result<(), Error> {
                        self.peaks[self.curr_peak_index]
                            // try to append item to the current peak
                            .try_append(input)
                            // if couldn't append, it's because the underlying tree is full
                            .or_else(|_| {
                                // so if the current peak is not last...
                                if self.curr_peak_index < #num_of_peaks - 1 {
                                    // move to the next peak and set it the new current one
                                    self.curr_peak_index += 1;
                                } else {
                                    // try to augment the last peak
                                    self.peaks[self.curr_peak_index] = self.peaks[self.curr_peak_index].try_augment()?;
                                }
                                // now try append the item to the new peak
                                self.peaks[self.curr_peak_index].try_append(input)
                            })
                            .map(|_| {
                                // now back propagate the peaks and merge them if necessary
                                self.merge_collapse();

                                let root = self.peaks[self.curr_peak_index].root();
                                self.summit_tree.replace_leaf(self.curr_peak_index, root);

                                self.num_of_leaves += 1;
                            })
                    }

                    // panics if the index is out of bounds
                    pub fn generate_proof(&mut self, index: usize) -> #mmr_proof_type {
                        let mut accrue_len = 0;
                        let mut i = 0;
                        // find the peak corresponding to the index
                        while accrue_len + self.peaks[i].num_of_leaves() <= index {
                            accrue_len += self.peaks[i].num_of_leaves();
                            i += 1;
                        }
                        // make thy entire proof by chaining the peak proof
                        // to the upper tree proof
                        chain_proofs(
                            self.peaks[i].generate_proof(index - accrue_len),
                            self.summit_tree.generate_proof(i)
                        )
                    }

                    pub fn base_layer_size(&self) -> usize {
                        self.peaks.iter().map(|peak| peak.base_layer_size()).sum()
                    }
                    pub fn num_of_leaves(&self) -> usize {
                        self.num_of_leaves
                    }
                    pub fn curr_peak_index(&self) -> usize {
                        self.curr_peak_index
                    }
                    pub fn peaks(&self) -> &[#mmr_peak_type] {
                        &self.peaks[..]
                    }
                }

                impl Default for #mmr_type
                where
                    [(); #num_of_peaks]: Sized,
                    {
                        fn default() -> Self {
                            Self::from_peak(Default::default())
                        }
                    }
            }

            use #mod_ident::#mmr_type as #mmr_type;
            use #mod_ident::#mmr_peak_type as #mmr_peak_type;
        };

    proc_macro::TokenStream::from(output)
}
