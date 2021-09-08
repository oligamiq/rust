//! Writing of the rustc metadata for dylibs

use rustc_metadata::creader::CStore;
use rustc_middle::ty::TyCtxt;

use crate::backend::WriteMetadata;

// Adapted from https://github.com/rust-lang/rust/blob/da573206f87b5510de4b0ee1a9c044127e409bd3/src/librustc_codegen_llvm/base.rs#L47-L112
pub(crate) fn write_metadata<O: WriteMetadata>(
    tcx: TyCtxt<'_>,
    object: &mut O,
    metadata: EncodedMetadata,
) {
    object.add_rustc_section(
        rustc_middle::middle::exported_symbols::metadata_symbol_name(tcx),
        metadata.compressed_metadata(),
    );
}
