use std::fmt::Debug;

use rustc_target::abi::Size;

#[derive(Copy, Clone)]
pub enum IntPredicate {
    IntEQ,
    IntNE,
    IntUGT,
    IntUGE,
    IntULT,
    IntULE,
    IntSGT,
    IntSGE,
    IntSLT,
    IntSLE,
}

pub trait Builder {
    type Value: Copy + Clone + Debug;
    type Type: Copy + Clone + Debug;

    fn type_of_val(&self, val: Self::Value) -> Self::Type;
    fn type_size(&self, ty: Self::Type) -> Size;

    fn const_null(&self, t: Self::Type) -> Self::Value;
    fn const_undef(&self, t: Self::Type) -> Self::Value;
    fn const_int(&self, t: Self::Type, i: i64) -> Self::Value;
    fn const_uint(&self, t: Self::Type, i: u64) -> Self::Value;
    fn const_uint_big(&self, t: Self::Type, u: u128) -> Self::Value;
    fn const_bool(&self, val: bool) -> Self::Value;
    fn const_i16(&self, i: i16) -> Self::Value;
    fn const_i32(&self, i: i32) -> Self::Value;
    fn const_u32(&self, i: u32) -> Self::Value;
    fn const_u64(&self, i: u64) -> Self::Value;
    fn const_usize(&self, i: u64) -> Self::Value;
    fn const_u8(&self, i: u8) -> Self::Value;

    // FIXME has side-effect. maybe remove?
    fn abort(&mut self);

    fn add(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn sub(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn mul(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn udiv(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn exactudiv(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn sdiv(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn exactsdiv(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn urem(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn srem(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn shl(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn lshr(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn ashr(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn unchecked_sadd(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn unchecked_uadd(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn unchecked_ssub(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn unchecked_usub(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn unchecked_smul(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn unchecked_umul(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn and(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn or(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn xor(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn neg(&mut self, v: Self::Value) -> Self::Value;
    fn not(&mut self, v: Self::Value) -> Self::Value;

    fn trunc(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value;
    fn zext(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value;
    fn sext(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value;

    // FIXME redundant with trunc/zext/sext?
    fn intcast(&mut self, val: Self::Value, dest_ty: Self::Type, is_signed: bool) -> Self::Value;

    fn ptraddr(&mut self, val: Self::Value) -> Self::Value;

    fn icmp(&mut self, op: IntPredicate, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn select(
        &mut self,
        cond: Self::Value,
        then_val: Self::Value,
        else_val: Self::Value,
    ) -> Self::Value;
}
