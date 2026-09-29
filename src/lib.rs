use ::core::{
    marker::PhantomData as PD,
    mem::{
        self,
        ManuallyDrop as MD,
        MaybeUninit as MU,
    },
    ptr,
};

use prelude::*;
pub
mod prelude {
    pub use crate::{
        brand::{Brand, MaybeBrand, Unbranded, brands},
        maybe_dropped::MaybeDropped,
        own_ref::OwnRef,
        out_ref::{BrandedPlace, OutRef, branded_places},
    };
}

pub 
mod brand;

pub 
mod hlist;

pub
mod maybe_dropped;

pub
mod own_ref;

pub
mod out_ref;
