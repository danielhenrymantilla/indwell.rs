// private prelude:
use {
    crate::{
        // extends public one.
        prelude::*,

        brand::NoBrandedPlace,
    },
    ::core::{
        marker::PhantomData as PD,
        mem::{
            self,
            ManuallyDrop as MD,
            MaybeUninit as MU,
        },
        ptr,
    },
};

pub
mod prelude;

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

use ::proc_macros;

mod utils;
