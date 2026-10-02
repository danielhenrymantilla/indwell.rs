use ::core::{
    marker::PhantomData as PD,
    mem::{
        self,
        ManuallyDrop as MD,
        MaybeUninit as MU,
    },
    ptr,
};
use ::proc_macros;

use prelude::*;
pub
mod prelude {
    pub use crate::{
        brand::{BrandedPlace, IsBrandedPlace, MaybeBrandedPlace, NoBrandedPlace, new_branded_place},
        hlist::HList,
        maybe_dropped::MaybeDropped,
        own_ref::OwnRef,
        out_ref::{BrandedUninit, OutRef, OutRefProject, branded_uninit},
    };
}

pub
mod brand;

pub
mod hlist;

pub
mod indwell;

pub
mod maybe_dropped;

pub
mod own_ref;

pub
mod out_ref;

mod utils;
