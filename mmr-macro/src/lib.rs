use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{LitInt, Ident, Token, Error};
use convert_case::{Case, Casing};

struct MMRInput {
    mmr_type: String,
    num_of_peaks: usize,
    branch_factor: usize,
    hash_type: String,
}

impl MMRInput {
    const TYPE_IDENT: &str = "Type";
    const BRANCH_FACTOR_IDENT: &str = "BranchFactor";
    const NUM_OF_PEAKS: &str = "Peaks";
    const HASH_TYPE_IDENT: &str = "Hash";
}

impl Parse for MMRInput {

    fn parse(input: ParseStream) -> Result<Self> {
        let mut with_type = false; 
        let maybe_type_ident = input.parse::<Ident>()?; 
        let mmr_type: String;
        if Self::TYPE_IDENT == &maybe_type_ident.to_string() {
            with_type = true;
            input.parse::<Token![=]>()?;
            mmr_type = input.parse::<Ident>()?.to_string();
            input.parse::<Token![,]>()?;
        } else {
            mmr_type = "MerkleMountainRange".to_owned();
        }   

        let branch_factor_ident = if with_type {
            input.parse::<Ident>()? 
        } else {
            maybe_type_ident
        };

        if Self::BRANCH_FACTOR_IDENT != &branch_factor_ident.to_string() {
            return Err(Error::new(branch_factor_ident.span(), &format!("expected {}", Self::BRANCH_FACTOR_IDENT)));
        }        
        input.parse::<Token![=]>()?;
        let branch_factor: LitInt = input.parse()?;
        
        input.parse::<Token![,]>()?;

        let num_of_peaks_ident = input.parse::<Ident>()?;  
        if Self::NUM_OF_PEAKS != &num_of_peaks_ident.to_string() {
            return Err(Error::new(num_of_peaks_ident.span(), &format!("expected {}", Self::NUM_OF_PEAKS)));
        }        
        input.parse::<Token![=]>()?;
        let num_of_peaks: LitInt = input.parse()?;

        input.parse::<Token![,]>()?;

        let hash_type_ident = input.parse::<Ident>()?;  
        if Self::HASH_TYPE_IDENT != &hash_type_ident.to_string() {
            return Err(Error::new(hash_type_ident.span(), &format!("expected {}", Self::HASH_TYPE_IDENT)));
        }        
        input.parse::<Token![=]>()?;
        let hash_type = input.parse::<Ident>()?.to_string();

        Ok(
            Self {
                mmr_type,
                num_of_peaks: num_of_peaks.base10_parse::<usize>()?,
                branch_factor: branch_factor.base10_parse::<usize>()?,
                hash_type,
            }
        )
    }
}

#[proc_macro]
pub fn mmr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as MMRInput);
    
    let peak_height = input.num_of_peaks;
    let summit_height = (8 * core::mem::size_of::<usize>() as u32 - input.num_of_peaks.leading_zeros()) as usize + 1;
    let total_height = summit_height + peak_height;

    let total_height = LitInt::new(&total_height.to_string(), proc_macro2::Span::call_site());
    let summit_height = LitInt::new(&summit_height.to_string(), proc_macro2::Span::call_site());
    let num_of_peaks = LitInt::new(&input.num_of_peaks.to_string(), proc_macro2::Span::call_site());
    let mmr_type = syn::Ident::new(&input.mmr_type, proc_macro2::Span::call_site());
    let mmr_peak_type = syn::Ident::new(&format!("{}Peak", input.mmr_type), proc_macro2::Span::call_site());
    let hash_type = syn::Ident::new(&input.hash_type, proc_macro2::Span::call_site());
    let mod_ident = syn::Ident::new(&input.mmr_type.to_case(Case::Snake), proc_macro2::Span::call_site());  

    let peak_variant_def_idents = (0..input.num_of_peaks)
        .map(|i| { 
            (
                syn::Ident::new(&format!("Peak{i}"), proc_macro2::Span::call_site()),
                LitInt::new(&(input.num_of_peaks - i).to_string(), proc_macro2::Span::call_site()),
            )
        })
        .collect::<Vec<(syn::Ident, LitInt)>>();

    let branch_factor = LitInt::new(&input.branch_factor.to_string(), proc_macro2::Span::call_site());
    
    let peak_variant_def_tokens = peak_variant_def_idents.iter()
        .map(|(peak_lit, peak_height)| { 
            quote! {
                #peak_lit(MergeableHeaplessTree<#branch_factor, #peak_height, #hash_type, PeakProof>)
            }
        })
        .collect::<Vec<_>>();

    let clone_impl_variant_def_tokens = peak_variant_def_idents.iter()
        .map(|(peak_lit, _)| { 
            quote! {
                #peak_lit(tree) => #peak_lit(tree.clone())
            }
        })
        .collect::<Vec<_>>();

    let default_variant_def_token = peak_variant_def_idents.iter().last()
        .map(|(peak_lit, _)| {
            quote! {
                #peak_lit(MergeableHeaplessTree::default())
            }
        })
        .expect("variant list is not empty. qed");

    let mut it1 = peak_variant_def_idents.iter().map(|(peak_lit, _)| peak_lit);
    let it2 = peak_variant_def_idents.iter().map(|(peak_lit, _)| peak_lit);
    it1.next();
    
    let try_merge_variant_def_tokens = it1.zip(it2)
        .map(|(peak_lit1, peak_lit2)| { 
            quote! {
                (#peak_lit1(this), #peak_lit1(other)) => this.try_merge(other).map_err(#peak_lit1).map(#peak_lit2)
            }
        })
        .collect::<Vec<_>>();


    let as_dyn_tree_variant_def_token = peak_variant_def_idents.iter()
        .map(|(peak_lit, _)| {
            quote! {
                #peak_lit(tree) => tree as &dyn BasicTreeTrait<#hash_type, PeakProof>
            }
        })
        .collect::<Vec<_>>();

    let as_mut_dyn_tree_variant_def_token = peak_variant_def_idents.iter()
        .map(|(peak_lit, _)| {
            quote! {
                #peak_lit(tree) => tree as &mut dyn BasicTreeTrait<#hash_type, PeakProof>
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

    let output = quote! {
        mod #mod_ident {            
            use merkle_heapless::{HashT, Proof, BasicTreeTrait, HeaplessTree, merge_proofs};
            use merkle_heapless::mergeable::mergeable::{MergeableHeaplessTree};
            use super::#hash_type;

            type PeakProof = Proof<#branch_factor, #peak_height, #hash_type>;
            type MMRProof = Proof<#branch_factor, #total_height, #hash_type>;

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
                pub fn try_merge(self, other: Self) -> Result<Self, Self> {
                    use #mmr_peak_type::{*};
                    match (self, other) {
                        #(#try_merge_variant_def_tokens),*,
                        _ => unreachable!(),
                    }
                }
            }        

            impl BasicTreeTrait<#hash_type, PeakProof> for #mmr_peak_type {
                fn generate_proof(&mut self, index: usize) -> PeakProof {
                    #impl_mut_method_body_token.generate_proof(index)
                }
                fn replace(&mut self, index: usize, input: &[u8]) {
                    #impl_mut_method_body_token.replace(index, input)
                }
                fn replace_leaf(&mut self, index: usize, leaf: <#hash_type as HashT>::Output) {
                    #impl_mut_method_body_token.replace_leaf(index, leaf)
                }
                fn remove(&mut self, index: usize) {
                    #impl_mut_method_body_token.remove(index)
                }
                fn try_append(&mut self, input: &[u8]) -> Result<(), ()> {
                    #impl_mut_method_body_token.try_append(input)
                }
                fn root(&self) -> <#hash_type as HashT>::Output {
                    #impl_method_body_token.root()
                }
                fn leaves(&self) -> &[<#hash_type as HashT>::Output] {
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
                fn num_of_leaves(&self) -> usize {
                    #impl_method_body_token.num_of_leaves()
                }
            }   

            pub struct #mmr_type 
            where 
                [(); #num_of_peaks]: Sized,
            {
                summit_tree: HeaplessTree<#branch_factor, #summit_height, #hash_type>,
                peaks: [#mmr_peak_type; #num_of_peaks],
                curr_peak_index: usize,
                num_of_leaves: usize,
            }

            impl #mmr_type 
            where 
                [(); #num_of_peaks]: Sized,
            {            
                pub fn from(peak: #mmr_peak_type) -> Result<Self, ()> {
                    let mut this = Self {
                        summit_tree: HeaplessTree::<#branch_factor, #summit_height, #hash_type>::try_from(&[])?,
                        peaks: [#mmr_peak_type::default(); #num_of_peaks],
                        curr_peak_index: 0,
                        num_of_leaves: 0,
                    }; 
                    this.peaks[0] = peak;
                    Ok(this)
                } 
                
                fn merge_collapse(&mut self) -> Result<(), ()> {
                    let mut i = self.curr_peak_index;
                    // back propagate and merge peaks while possible    
                    // the indicator that two peaks can merge is that they have the same rank (can check height or num_of_leaves)
                    while i > 0 && self.peaks[i].height() == self.peaks[i - 1].height() {
                        if self.peaks[i - 1]
                                .try_merge(self.peaks[i])
                                .map(|merged| {
                                    self.peaks[i - 1] = merged;
                                    self.peaks[i] = Default::default();
                                }).is_err() {
                            return Err(());
                        }  
                        i -= 1;
                        self.curr_peak_index = i;
                    }
                    Ok(())
                }    

                pub fn try_append(&mut self, input: &[u8]) -> Result<(), ()> {
                    let prev_peak_index = self.curr_peak_index;
                    
                    self.peaks[self.curr_peak_index]
                        // try to append item to the current peak
                        .try_append(input)
                        // if couldn't append, it's because the underlying tree is full
                        .or_else(|_| {
                            // so if the current peak is not last...
                            if self.curr_peak_index < #num_of_peaks {
                                // move to the next peak and set it the new current one
                                self.curr_peak_index += 1;
                                // try append the item now to the new peak
                                self.peaks[self.curr_peak_index].try_append(input)
                            } else { 
                                Err(())
                            }
                        })
                        .and_then(|_| {
                            let need_to_rebuild_summit_tree = prev_peak_index != self.curr_peak_index;      
                            // now back propagate the peaks and merge them if necessary
                            self.merge_collapse().map(|_| {
                                if need_to_rebuild_summit_tree {
                                    for (i, peak) in self.peaks.iter().enumerate() { 
                                        self.summit_tree.replace_leaf(i, peak.root())
                                    }
                                } else {
                                    let i = self.curr_peak_index;
                                    self.summit_tree.replace_leaf(i, self.peaks[i].root());
                                }
                                self.num_of_leaves += 1;
                            })
                        })
                }

                pub fn generate_proof(&mut self, index: usize) -> MMRProof {
                    let mut accum_len = 0;
                    let mut peak_ind = 0;
            
                    for peak in self.peaks.iter() {
                        if accum_len + peak.num_of_leaves() > index {
                            break;
                        }
                        peak_ind += 1;
                        accum_len += peak.num_of_leaves();
                    }
                    merge_proofs(
                        self.peaks[peak_ind].generate_proof(index - accum_len),
                        self.summit_tree.generate_proof(peak_ind)
                    )
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
                        Self::from(Default::default()).unwrap()
                    }
                }
        }

        use #mod_ident::#mmr_type as #mmr_type;
        use #mod_ident::#mmr_peak_type as #mmr_peak_type;
    };
    
    proc_macro::TokenStream::from(output)
}
