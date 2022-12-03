use rustc_target::abi::{HasDataLayout, Primitive, Scalar, TagEncoding, VariantIdx, Variants};

use crate::builder::{Builder, IntPredicate};

/// Obtain the actual discriminant of a value.
#[instrument(level = "trace", skip(bx, cx))]
pub fn decode_tag_to_discr<Bx: Builder>(
    bx: &mut Bx,
    cx: &impl HasDataLayout,
    tag_imm: Bx::Value,
    tag_scalar: Scalar,
    tag_encoding: TagEncoding<VariantIdx>,
    cast_to: Bx::Type,
) -> Bx::Value {
    // Decode the discriminant (specifically if it's niche-encoded).
    match tag_encoding {
        TagEncoding::Direct => {
            let signed = match tag_scalar.primitive() {
                // We use `i1` for bytes that are always `0` or `1`,
                // e.g., `#[repr(i8)] enum E { A, B }`, but we can't
                // let LLVM interpret the `i1` as signed, because
                // then `i1 1` (i.e., `E::B`) is effectively `i8 -1`.
                Primitive::Int(_, signed) => !tag_scalar.is_bool() && signed,
                _ => false,
            };
            bx.intcast(tag_imm, cast_to, signed)
        }
        TagEncoding::Niche { untagged_variant, ref niche_variants, niche_start } => {
            // Cast to an integer so we don't have to treat a pointer as a
            // special case.
            let tag = if tag_scalar.primitive().is_ptr() { bx.ptraddr(tag_imm) } else { tag_imm };
            let tag_llty = bx.type_of_val(tag);

            let tag_size = tag_scalar.size(cx);
            let max_unsigned = tag_size.unsigned_int_max();
            let max_signed = tag_size.signed_int_max() as u128;
            let min_signed = max_signed + 1;
            let relative_max = niche_variants.end().as_u32() - niche_variants.start().as_u32();
            let niche_end = niche_start.wrapping_add(relative_max as u128) & max_unsigned;
            let range = tag_scalar.valid_range(cx);

            let sle = |lhs: u128, rhs: u128| -> bool {
                // Signed and unsigned comparisons give the same results,
                // except that in signed comparisons an integer with the
                // sign bit set is less than one with the sign bit clear.
                // Toggle the sign bit to do a signed comparison.
                (lhs ^ min_signed) <= (rhs ^ min_signed)
            };

            // We have a subrange `niche_start..=niche_end` inside `range`.
            // If the value of the tag is inside this subrange, it's a
            // "niche value", an increment of the discriminant. Otherwise it
            // indicates the untagged variant.
            // A general algorithm to extract the discriminant from the tag
            // is:
            // relative_tag = tag - niche_start
            // is_niche = relative_tag <= (ule) relative_max
            // discr = if is_niche {
            //     cast(relative_tag) + niche_variants.start()
            // } else {
            //     untagged_variant
            // }
            // However, we will likely be able to emit simpler code.

            // Find the least and greatest values in `range`, considered
            // both as signed and unsigned.
            let (low_unsigned, high_unsigned) =
                if range.start <= range.end { (range.start, range.end) } else { (0, max_unsigned) };
            let (low_signed, high_signed) = if sle(range.start, range.end) {
                (range.start, range.end)
            } else {
                (min_signed, max_signed)
            };

            let niches_ule = niche_start <= niche_end;
            let niches_sle = sle(niche_start, niche_end);
            let cast_smaller = bx.type_size(cast_to) <= tag_size;

            // In the algorithm above, we can change
            // cast(relative_tag) + niche_variants.start()
            // into
            // cast(tag) + (niche_variants.start() - niche_start)
            // if either the casted type is no larger than the original
            // type, or if the niche values are contiguous (in either the
            // signed or unsigned sense).
            let can_incr_after_cast = cast_smaller || niches_ule || niches_sle;

            let data_for_boundary_niche = || -> Option<(IntPredicate, u128)> {
                if !can_incr_after_cast {
                    None
                } else if niche_start == low_unsigned {
                    Some((IntPredicate::IntULE, niche_end))
                } else if niche_end == high_unsigned {
                    Some((IntPredicate::IntUGE, niche_start))
                } else if niche_start == low_signed {
                    Some((IntPredicate::IntSLE, niche_end))
                } else if niche_end == high_signed {
                    Some((IntPredicate::IntSGE, niche_start))
                } else {
                    None
                }
            };

            let (is_niche, tagged_discr, delta) = if relative_max == 0 {
                // Best case scenario: only one tagged variant. This will
                // likely become just a comparison and a jump.
                // The algorithm is:
                // is_niche = tag == niche_start
                // discr = if is_niche {
                //     niche_start
                // } else {
                //     untagged_variant
                // }
                let niche_start = bx.const_uint_big(tag_llty, niche_start);
                let is_niche = bx.icmp(IntPredicate::IntEQ, tag, niche_start);
                let tagged_discr = bx.const_uint(cast_to, niche_variants.start().as_u32() as u64);
                (is_niche, tagged_discr, 0)
            } else if let Some((predicate, constant)) = data_for_boundary_niche() {
                // The niche values are either the lowest or the highest in
                // `range`. We can avoid the first subtraction in the
                // algorithm.
                // The algorithm is now this:
                // is_niche = tag <= niche_end
                // discr = if is_niche {
                //     cast(tag) + (niche_variants.start() - niche_start)
                // } else {
                //     untagged_variant
                // }
                // (the first line may instead be tag >= niche_start,
                // and may be a signed or unsigned comparison)
                let is_niche = bx.icmp(predicate, tag, bx.const_uint_big(tag_llty, constant));
                let cast_tag = if cast_smaller {
                    bx.intcast(tag, cast_to, false)
                } else if niches_ule {
                    bx.zext(tag, cast_to)
                } else {
                    bx.sext(tag, cast_to)
                };

                let delta = (niche_variants.start().as_u32() as u128).wrapping_sub(niche_start);
                (is_niche, cast_tag, delta)
            } else {
                // The special cases don't apply, so we'll have to go with
                // the general algorithm.
                let relative_discr = bx.sub(tag, bx.const_uint_big(tag_llty, niche_start));
                let cast_tag = bx.intcast(relative_discr, cast_to, false);
                let is_niche = bx.icmp(
                    IntPredicate::IntULE,
                    relative_discr,
                    bx.const_uint(tag_llty, relative_max as u64),
                );
                (is_niche, cast_tag, niche_variants.start().as_u32() as u128)
            };

            let tagged_discr = if delta == 0 {
                tagged_discr
            } else {
                bx.add(tagged_discr, bx.const_uint_big(cast_to, delta))
            };

            let discr = bx.select(
                is_niche,
                tagged_discr,
                bx.const_uint(cast_to, untagged_variant.as_u32() as u64),
            );

            // In principle we could insert assumes on the possible range of `discr`, but
            // currently in LLVM this seems to be a pessimization.

            discr
        }
    }
}

/// Sets the discriminant for a new value of the given case of the given
/// representation.
pub fn encode_tag_from_discr<Bx: Builder>(
    bx: &mut Bx,
    cx: &impl HasDataLayout,
    ty: Ty<'_>,
    variant_index: VariantIdx,
    tag_encoding: TagEncoding<VariantIdx>,
) -> Option<Bx::Value> {
    match tag_encoding {
        TagEncoding::Direct => {
            let to = ty.discriminant_for_variant(cx, variant_index).unwrap().val;
            Some(bx.const_uint_big(bx.cx().backend_type(ptr.layout), to))
        }
        TagEncoding::Niche { untagged_variant, ref niche_variants, niche_start } => {
            if variant_index == untagged_variant {
                None
            } else {
                let niche_llty = bx.cx().immediate_backend_type(niche.layout);
                let niche_value = variant_index.as_u32() - niche_variants.start().as_u32();
                let niche_value = (niche_value as u128).wrapping_add(niche_start);
                // FIXME(eddyb): check the actual primitive type here.
                let niche_llval = if niche_value == 0 {
                    // HACK(eddyb): using `c_null` as it works on all types.
                    bx.const_null(niche_llty)
                } else {
                    bx.const_uint_big(niche_llty, niche_value)
                };
                Some(niche_llval)
            }
        }
    }
}
