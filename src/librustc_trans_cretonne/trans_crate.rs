use rustc_data_structures::indexed_vec::Idx;
use rustc::dep_graph::DepGraph;
use rustc::hir::def_id::CrateNum;
use rustc::middle::cstore::MetadataLoader;
use rustc::middle::cstore::{CrateSource, LibSource, NativeLibrary};
use rustc::session::Session;
use rustc::session::config::{OutputFilenames, OutputType};
use rustc::ty::maps::Providers;
use rustc::ty::{self, Ty, TyCtxt, TypeVariants};
use rustc::util::nodemap::{FxHashMap, FxHashSet};
use rustc::middle::trans::TransItem;
use rustc::mir::*;

use cretonne::ir::function::Function;
use cretonne::ir::{ArgumentType, CallConv, Signature};
use cretonne::ir::types::Type;
use cton_frontend::{FunctionBuilder, ILBuilder};

#[derive(Eq, PartialEq, Copy, Clone)]
struct Local(u32);

impl ::cretonne::entity::EntityRef for Local {
    fn new(index: usize) -> Self {
        debug_assert!(index < (::std::u32::MAX as usize));
        Local(index as u32)
    }

    fn index(self) -> usize {
        self.0 as usize
    }
}

impl Default for Local {
    fn default() -> Self {
        Local(::std::u32::MAX)
    }
}

pub fn trans_crate<'a, 'tcx: 'a>(tcx: TyCtxt<'a, 'tcx, 'tcx>) -> () {
    let mut il_builder = ILBuilder::<Local>::new();
    let (trans_items, _inlining_map) = ::collector::collect_crate_translation_items(
        tcx,
        ::collector::TransItemCollectionMode::Eager,
    );
    for trans_item in trans_items {
        match trans_item {
            TransItem::Fn(instance) => {
                let mir = tcx.instance_mir(instance.def);
                let _ = trans_function(tcx, &mut il_builder, mir);
            }
            _ => unimplemented!("Unsupported"),
        }
    }
}

fn trans_function(tcx: TyCtxt, il_builder: &mut ILBuilder<Local>, mir: &Mir) -> Result<(), ()> {
    let mut func = Function::new();
    let mut sig = Signature::new(CallConv::Native);

    sig.return_types = vec![ArgumentType::new(rust_ty_to_cretonne_ty(mir.return_ty)?)];
    sig.argument_types = mir.local_decls
        .iter()
        .map(|decl| {
            Ok(ArgumentType::new(rust_ty_to_cretonne_ty(decl.ty)?))
        })
        .collect::<Result<Vec<_>, ()>>()
        .map_err(|_| tcx.sess.span_err(mir.span, "Unsupported type"))?;
    func.signature = sig.clone();

    let mut builder = FunctionBuilder::new(&mut func, il_builder);
    let entry_block = builder.create_ebb();
    builder.switch_to_block(entry_block, &[]);
    builder.seal_block(entry_block);
    builder.ensure_inserted_ebb();

    for mir_bb in mir.basic_blocks() {
        let ebb = builder.create_ebb();
    }
    tcx.sess
        .struct_span_warn(
            mir.span,
            &format!(
                "{:?}",
                mir.local_decls.iter().map(|d| d.ty).collect::<Vec<_>>()
            ),
        )
        .note(&format!("{:?}", sig))
        .note(&format!("{}", builder.display(None)))
        .emit();
    Ok(())
}

fn rust_ty_to_cretonne_ty(ty: Ty) -> Result<Type, ()> {
    use syntax::ast::{FloatTy, IntTy, UintTy};
    use rustc::ty::TypeVariants as TyVar;
    use cretonne::ir::types::*;
    let Isize = I64;
    Ok(match ty.sty {
        TyVar::TyBool => B1,
        TyVar::TyChar => I32,
        TyVar::TyInt(IntTy::I8) => I8,
        TyVar::TyInt(IntTy::I16) => I16,
        TyVar::TyInt(IntTy::I32) => I32,
        TyVar::TyInt(IntTy::I64) => I64,
        TyVar::TyInt(IntTy::I128) => {
            tcx.sess.span_err(mir.span, "Unsupported type i128");
            Err(())?
        }
        TyVar::TyInt(IntTy::Is) => Isize,
        TyVar::TyUint(UintTy::U8) => I8,
        TyVar::TyUint(UintTy::U16) => I16,
        TyVar::TyUint(UintTy::U32) => I32,
        TyVar::TyUint(UintTy::U64) => I64,
        TyVar::TyUint(UintTy::U128) => {
            tcx.sess.span_err(mir.span, "Unsupported type u128");
            Err(())?
        }
        TyVar::TyUint(UintTy::Us) => Isize,
        TyVar::TyFloat(FloatTy::F32) => F32,
        TyVar::TyFloat(FloatTy::F64) => F64,
        TyAdt(adt_def, _) => Isize,
        TyVar::TyRef(_, _) => Isize,
        TyVar::TyNever => VOID,
        TyVar::TyTuple(slice, _) => if slice.is_empty() {
            VOID
        } else {
            unimplemented!("slice: {:?}", slice);
        },
        _ => unimplemented!("{:?}", ty),
    })
}
