use crate::MirPass;

use rustc_index::vec::Idx;
use rustc_middle::mir::patch::MirPatch;
use rustc_middle::mir::{
    AggregateKind, BasicBlockData, Body, Constant, ConstantKind, Location, NonDivergingIntrinsic,
    Operand, Place, Rvalue, StatementKind, TerminatorKind,
};
use rustc_middle::ty::InternalSubsts;
use rustc_middle::ty::TyCtxt;
use rustc_target::abi::VariantIdx;

pub struct ValgrindClientRequest;

impl<'tcx> MirPass<'tcx> for ValgrindClientRequest {
    #[instrument(skip(self, tcx, body))]
    fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
        if !tcx.sess.opts.unstable_opts.instrument_krabcake {
            info!("Not instrumenting for krabcake");
            return;
        }
        info!("Instrumenting for krabcake now...");
        let mut patch = MirPatch::new(body);

        for (block_index, block_data) in body.basic_blocks.iter_enumerated() {
            for (stmt_index, stmt) in block_data.statements.iter().enumerate() {
                match &stmt.kind {
                    StatementKind::Intrinsic(box NonDivergingIntrinsic::Assume(operand)) => {
                        let loc = Location { block: block_index, statement_index: stmt_index };
                        info!("Found assume intrinsic (operand={operand:?}. At {loc:?}");
                        patch = call_ikr(tcx, patch, body, loc, operand);
                    }
                    _ => (),
                }
            }
        }

        patch.apply(body);
    }
}

fn call_ikr<'tcx>(
    tcx: TyCtxt<'tcx>,
    mut patch: MirPatch<'tcx>,
    body: &Body<'tcx>,
    loc: Location,
    _operand: &Operand<'tcx>,
) -> MirPatch<'tcx> {
    let span = patch.source_info_for_location(body, loc).span;

    let op = |flag: bool| {
        Operand::Constant(Box::new(Constant {
            span,
            user_ty: None,
            literal: ConstantKind::from_bool(tcx, flag),
        }))
    };

    let (place, rvalue) = {
        let krabcake_req_did = tcx.lang_items().krabcake_request().unwrap();
        let krabcake_req_substs = InternalSubsts::identity_for_item(tcx, krabcake_req_did);
        let krabcake_req_def = tcx.adt_def(krabcake_req_did);
        let krabcake_req_ty = tcx.mk_adt(krabcake_req_def, krabcake_req_substs);
        let rvalue = Rvalue::Aggregate(
            Box::new(AggregateKind::Adt(
                krabcake_req_did,
                VariantIdx::new(0),
                &krabcake_req_substs,
                None,
                None,
            )),
            vec![op(true)],
        );
        let temp = patch.new_temp(krabcake_req_ty, span);
        let place = Place::from(temp);
        (place, rvalue)
    };

    patch.add_assign(loc, place, rvalue);

    let krabcake_req_operand = Operand::Copy(place);

    let orig_terminator = patch.terminator_for_location(body, loc);
    let ikr_did = tcx.lang_items().insert_krabcake_request_fn().unwrap();
    let ikr_substs = InternalSubsts::identity_for_item(tcx, ikr_did);
    let ikr_ty = tcx.mk_fn_def(ikr_did, ikr_substs);

    let func = Operand::Constant(Box::new(Constant {
        span,
        user_ty: None,
        literal: ConstantKind::zero_sized(ikr_ty),
    }));
    let storage = patch.new_temp(tcx.mk_mut_ptr(tcx.types.unit), span);
    let storage = Place::from(storage);
    let fn_call_terminator_kind = TerminatorKind::Call {
        func,
        args: vec![krabcake_req_operand],
        destination: storage,
        target: Some(loc.block + 1),
        cleanup: None,
        from_hir_call: false,
        fn_span: span,
    };

    patch.patch_terminator(loc.block, fn_call_terminator_kind);

    let new_bb =
        BasicBlockData { statements: vec![], terminator: orig_terminator, is_cleanup: false };

    patch.new_block(new_bb);
    patch
}
