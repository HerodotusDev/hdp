/// See [`hdp_provider`] for more details.
#[cfg(feature = "provider")]
pub mod provider {
    #[doc(inline)]
    pub use hdp_provider::*;
}

/// See [`hdp_processor`] for more details.
#[cfg(feature = "processor")]
pub mod processor {
    #[doc(inline)]
    pub use hdp_processor::*;
}

/// See [`hdp_preprocessor`] for more details.
#[cfg(feature = "preprocessor")]
pub mod preprocessor {
    #[doc(inline)]
    pub use hdp_preprocessor::*;
}

/// See [`hdp_primitives`] for more details.
#[cfg(feature = "primitives")]
pub mod primitives {
    #[doc(inline)]
    pub use hdp_primitives::*;
}

/// See [`hdp_cairo_runner`] for more details.
#[cfg(feature = "cairo-runner")]
pub mod cairo_runner {
    #[doc(inline)]
    pub use hdp_cairo_runner::*;
}
