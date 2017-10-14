use std::marker::PhantomData;
use std::cell::Cell;
use std::sync::Arc;
use rustc::traits;
use rustc::ty::{self, Ty, TyCtxt};
use rustc::ty::layout::{HasDataLayout, LayoutCx, LayoutError, LayoutTyper, TyLayout};
use rustc::dep_graph::DepGraph;
use rustc::session::Session;
use rustc::middle::trans::CodegenUnit;

use common;

/// The shared portion of a `CrateContext`.  There is one `SharedCrateContext`
/// per crate.  The data here is shared between all compilation units of the
/// crate, so it must not contain references to any LLVM data structures
/// (aside from metadata-related ones).
pub struct SharedCrateContext<'a, 'tcx: 'a> {
    tcx: TyCtxt<'a, 'tcx, 'tcx>,
    check_overflow: bool,
    use_dll_storage_attrs: bool,
}


/// The local portion of a `CrateContext`.  There is one `LocalCrateContext`
/// per compilation unit.  Each one has its own LLVM `ContextRef` so that
/// several compilation units may be optimized in parallel.  All other LLVM
/// data structures in the `LocalCrateContext` are tied to that `ContextRef`.
pub struct LocalCrateContext<'a, 'tcx: 'a> {
    codegen_unit: Arc<CodegenUnit<'tcx>>,
    /// A counter that is used for generating local symbol names
    local_gen_sym_counter: Cell<usize>,

    /// A placeholder so we can add lifetimes
    placeholder: PhantomData<&'a ()>,
}

/// A CrateContext value binds together one LocalCrateContext with the
/// SharedCrateContext. It exists as a convenience wrapper, so we don't have to
/// pass around (SharedCrateContext, LocalCrateContext) tuples all over trans.
pub struct CrateContext<'a, 'tcx: 'a> {
    shared: &'a SharedCrateContext<'a, 'tcx>,
    local_ccx: &'a LocalCrateContext<'a, 'tcx>,
}

impl<'a, 'tcx> CrateContext<'a, 'tcx> {
    pub fn new(shared: &'a SharedCrateContext<'a, 'tcx>, local_ccx: &'a LocalCrateContext<'a, 'tcx>) -> Self {
        CrateContext { shared, local_ccx }
    }
}

impl<'b, 'tcx> SharedCrateContext<'b, 'tcx> {
    pub fn new(tcx: TyCtxt<'b, 'tcx, 'tcx>) -> SharedCrateContext<'b, 'tcx> {
        // An interesting part of Windows which MSVC forces our hand on (and
        // apparently MinGW didn't) is the usage of `dllimport` and `dllexport`
        // attributes in LLVM IR as well as native dependencies (in C these
        // correspond to `__declspec(dllimport)`).
        //
        // Whenever a dynamic library is built by MSVC it must have its public
        // interface specified by functions tagged with `dllexport` or otherwise
        // they're not available to be linked against. This poses a few problems
        // for the compiler, some of which are somewhat fundamental, but we use
        // the `use_dll_storage_attrs` variable below to attach the `dllexport`
        // attribute to all LLVM functions that are exported e.g. they're
        // already tagged with external linkage). This is suboptimal for a few
        // reasons:
        //
        // * If an object file will never be included in a dynamic library,
        //   there's no need to attach the dllexport attribute. Most object
        //   files in Rust are not destined to become part of a dll as binaries
        //   are statically linked by default.
        // * If the compiler is emitting both an rlib and a dylib, the same
        //   source object file is currently used but with MSVC this may be less
        //   feasible. The compiler may be able to get around this, but it may
        //   involve some invasive changes to deal with this.
        //
        // The flipside of this situation is that whenever you link to a dll and
        // you import a function from it, the import should be tagged with
        // `dllimport`. At this time, however, the compiler does not emit
        // `dllimport` for any declarations other than constants (where it is
        // required), which is again suboptimal for even more reasons!
        //
        // * Calling a function imported from another dll without using
        //   `dllimport` causes the linker/compiler to have extra overhead (one
        //   `jmp` instruction on x86) when calling the function.
        // * The same object file may be used in different circumstances, so a
        //   function may be imported from a dll if the object is linked into a
        //   dll, but it may be just linked against if linked into an rlib.
        // * The compiler has no knowledge about whether native functions should
        //   be tagged dllimport or not.
        //
        // For now the compiler takes the perf hit (I do not have any numbers to
        // this effect) by marking very little as `dllimport` and praying the
        // linker will take care of everything. Fixing this problem will likely
        // require adding a few attributes to Rust itself (feature gated at the
        // start) and then strongly recommending static linkage on MSVC!
        let use_dll_storage_attrs = tcx.sess.target.target.options.is_like_msvc;

        let check_overflow = tcx.sess.overflow_checks();

        SharedCrateContext {
            tcx,
            check_overflow,
            use_dll_storage_attrs,
        }
    }

    pub fn type_needs_drop(&self, ty: Ty<'tcx>) -> bool {
        common::type_needs_drop(self.tcx, ty)
    }

    pub fn type_is_sized(&self, ty: Ty<'tcx>) -> bool {
        common::type_is_sized(self.tcx, ty)
    }

    pub fn type_is_freeze(&self, ty: Ty<'tcx>) -> bool {
        common::type_is_freeze(self.tcx, ty)
    }

    pub fn tcx(&self) -> TyCtxt<'b, 'tcx, 'tcx> {
        self.tcx
    }

    pub fn sess<'a>(&'a self) -> &'a Session {
        &self.tcx.sess
    }

    pub fn dep_graph<'a>(&'a self) -> &'a DepGraph {
        &self.tcx.dep_graph
    }

    pub fn use_dll_storage_attrs(&self) -> bool {
        self.use_dll_storage_attrs
    }
}

impl<'a, 'tcx> LocalCrateContext<'a, 'tcx> {
    pub fn new(
        _shared: &SharedCrateContext<'a, 'tcx>,
        codegen_unit: Arc<CodegenUnit<'tcx>>,
        llmod_id: &str,
    ) -> LocalCrateContext<'a, 'tcx> {
        LocalCrateContext {
            local_gen_sym_counter: Cell::new(0),
            codegen_unit,
            placeholder: PhantomData,
        }
    }

    /// Create a dummy `CrateContext` from `self` and  the provided
    /// `SharedCrateContext`.  This is somewhat dangerous because `self` may
    /// not be fully initialized.
    ///
    /// This is used in the `LocalCrateContext` constructor to allow calling
    /// functions that expect a complete `CrateContext`, even before the local
    /// portion is fully initialized and attached to the `SharedCrateContext`.
    fn dummy_ccx(
        shared: &'a SharedCrateContext<'a, 'tcx>,
        local_ccxs: &'a [LocalCrateContext<'a, 'tcx>],
    ) -> CrateContext<'a, 'tcx> {
        assert!(local_ccxs.len() == 1);
        CrateContext {
            shared,
            local_ccx: &local_ccxs[0],
        }
    }
}

impl<'b, 'tcx> CrateContext<'b, 'tcx> {
    pub fn shared(&self) -> &'b SharedCrateContext<'b, 'tcx> {
        self.shared
    }

    fn local(&self) -> &'b LocalCrateContext<'b, 'tcx> {
        self.local_ccx
    }

    pub fn tcx(&self) -> TyCtxt<'b, 'tcx, 'tcx> {
        self.shared.tcx
    }

    pub fn sess<'a>(&'a self) -> &'a Session {
        &self.shared.tcx.sess
    }

    pub fn check_overflow(&self) -> bool {
        self.shared.check_overflow
    }

    pub fn use_dll_storage_attrs(&self) -> bool {
        self.shared.use_dll_storage_attrs()
    }

    pub fn codegen_unit(&self) -> &CodegenUnit<'tcx> {
        &self.local().codegen_unit
    }

    /// Generate a new symbol name with the given prefix. This symbol name must
    /// only be used for definitions with `internal` or `private` linkage.
    pub fn generate_local_symbol_name(&self, prefix: &str) -> String {
        use rustc_data_structures::base_n;

        let idx = self.local().local_gen_sym_counter.get();
        self.local().local_gen_sym_counter.set(idx + 1);
        // Include a '.' character, so there can be no accidental conflicts with
        // user defined names
        let mut name = String::with_capacity(prefix.len() + 6);
        name.push_str(prefix);
        name.push_str(".");
        base_n::push_str(idx as u64, base_n::ALPHANUMERIC_ONLY, &mut name);
        name
    }
}

impl<'a, 'tcx> HasDataLayout for &'a SharedCrateContext<'a, 'tcx> {
    fn data_layout(&self) -> &ty::layout::TargetDataLayout {
        &self.tcx.data_layout
    }
}

impl<'a, 'tcx> HasDataLayout for &'a CrateContext<'a, 'tcx> {
    fn data_layout(&self) -> &ty::layout::TargetDataLayout {
        &self.shared.tcx.data_layout
    }
}

impl<'a, 'tcx> LayoutTyper<'tcx> for &'a SharedCrateContext<'a, 'tcx> {
    type TyLayout = TyLayout<'tcx>;

    fn tcx<'b>(&'b self) -> TyCtxt<'b, 'tcx, 'tcx> {
        self.tcx
    }

    fn layout_of(self, ty: Ty<'tcx>) -> Self::TyLayout {
        let param_env = ty::ParamEnv::empty(traits::Reveal::All);
        LayoutCx::new(self.tcx, param_env)
            .layout_of(ty)
            .unwrap_or_else(|e| match e {
                LayoutError::SizeOverflow(_) => self.sess().fatal(&e.to_string()),
                _ => bug!("failed to get layout for `{}`: {}", ty, e)
            })
    }

    fn normalize_projections(self, ty: Ty<'tcx>) -> Ty<'tcx> {
        self.tcx().normalize_associated_type(&ty)
    }
}

impl<'a, 'tcx> LayoutTyper<'tcx> for &'a CrateContext<'a, 'tcx> {
    type TyLayout = TyLayout<'tcx>;

    fn tcx<'b>(&'b self) -> TyCtxt<'b, 'tcx, 'tcx> {
        self.shared.tcx
    }

    fn layout_of(self, ty: Ty<'tcx>) -> Self::TyLayout {
        self.shared.layout_of(ty)
    }

    fn normalize_projections(self, ty: Ty<'tcx>) -> Ty<'tcx> {
        self.shared.normalize_projections(ty)
    }
}
