use crate::MirPass;

// use rustc_ast::ast::{InlineAsmOptions, InlineAsmTemplatePiece};
// use rustc_ast::Mutability;
use rustc_middle::mir::patch::MirPatch;
// use rustc_middle::mir::Location;
use rustc_middle::mir::{
    /*AggregateKind,*/ BasicBlock, BasicBlockData, Body, Constant,
    ConstantKind, /*InlineAsmOperand,*/
    Operand, Place, /* Rvalue, */ TerminatorKind,
};
use rustc_middle::ty::InternalSubsts;
use rustc_middle::ty::TyCtxt;
// use rustc_span::Span;
// use rustc_target::asm::{InlineAsmReg, InlineAsmRegOrRegClass, X86InlineAsmReg};

pub struct ValgrindClientRequest;

impl<'tcx> MirPass<'tcx> for ValgrindClientRequest {
    #[instrument(skip(self, tcx, body))]
    fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
        if !tcx.sess.opts.unstable_opts.instrument_krabcake {
            info!("Not instrumenting for krabcake");
            return;
        }
        info!("Instrumenting for krabcake now...");
        let bbs = &body.basic_blocks;

        let target_block_index = BasicBlock::from_usize(1);
        // let statements_len = bbs[target_block_index].statements.len();
        let target_block = bbs.get(target_block_index).expect("No last block!!");

        let mut patch = MirPatch::new(body);
        // let target_block_end =
        //     Location { block: target_block_index, statement_index: statements_len };

        // let array_ty = tcx.mk_array(tcx.types.u64, 6);
        let span = target_block.terminator().source_info.span;
        // let array_place = add_assign_array(tcx, &mut patch, span, array_ty, target_block_end);
        // let array_raw_ptr_place = add_assign_array_raw_ptr(
        //     tcx,
        //     &mut patch,
        //     span,
        //     array_ty,
        //     target_block_end,
        //     array_place,
        // );
        // let zzq_result = add_assign_zzq_result(tcx, &mut patch, span, target_block_end);
        // patch_terminator_with_asm(
        //     tcx,
        //     &mut patch,
        //     array_raw_ptr_place,
        //     bbs.next_index(),
        //     target_block_index,
        //     zzq_result,
        // );

        let ikr_did = tcx.lang_items().insert_krabcake_request_fn().unwrap();
        let substs = InternalSubsts::identity_for_item(tcx, ikr_did);
        let ikr_ty = tcx.mk_fn_def(ikr_did, substs);

        let func = Operand::Constant(Box::new(Constant {
            span,
            user_ty: None,
            literal: ConstantKind::zero_sized(ikr_ty),
        }));
        let storage = patch.new_temp(tcx.mk_mut_ptr(tcx.types.unit), span);
        let storage = Place::from(storage);
        let fn_call_terminator_kind = TerminatorKind::Call {
            func,
            args: vec![],
            // args: vec![Operand::Move(ref_loc)],
            destination: storage,
            target: Some(bbs.next_index()),
            cleanup: None,
            // cleanup: Some(cleanup),
            from_hir_call: false, // Not sure
            fn_span: span,
        };

        patch.patch_terminator(target_block_index, fn_call_terminator_kind);

        let new_bb = BasicBlockData {
            statements: vec![],
            terminator: target_block.terminator.to_owned(),
            is_cleanup: false,
        };

        patch.new_block(new_bb);
        patch.apply(body);
    }
}

// fn add_assign_array<'tcx>(
//     tcx: TyCtxt<'tcx>,
//     patch: &mut MirPatch<'tcx>,
//     span: Span,
//     array_ty: Ty<'tcx>,
//     loc: Location,
// ) -> Place<'tcx> {
//     let lit = |n: u128| ConstantKind::from_bits(tcx, n, ParamEnv::empty().and(tcx.types.u64));
//     let op =
//         |n: u128| Operand::Constant(Box::new(Constant { span, user_ty: None, literal: lit(n) }));

//     let rvalue = Rvalue::Aggregate(
//         Box::new(AggregateKind::Array(tcx.types.u64)),
//         vec![op(0x4b430000), op(2), op(3), op(4), op(5), op(6)],
//     );
//     let temp = patch.new_temp(array_ty, span);
//     let place = Place::from(temp);
//     patch.add_assign(loc, place, rvalue);
//     place
// }

// fn add_assign_array_raw_ptr<'tcx>(
//     tcx: TyCtxt<'tcx>,
//     patch: &mut MirPatch<'tcx>,
//     span: Span,
//     array_ty: Ty<'tcx>,
//     loc: Location,
//     array_place: Place<'tcx>,
// ) -> Place<'tcx> {
//     let rvalue = Rvalue::AddressOf(Mutability::Not, array_place);
//     let ptr_ty = tcx.mk_ptr(TypeAndMut { ty: array_ty, mutbl: Mutability::Not });
//     let new_temp = patch.new_temp(ptr_ty, span);
//     let place = Place::from(new_temp);
//     patch.add_assign(loc, place, rvalue);
//     place
// }

// fn add_assign_zzq_result<'tcx>(
//     tcx: TyCtxt<'tcx>,
//     patch: &mut MirPatch<'tcx>,
//     span: Span,
//     loc: Location,
// ) -> Place<'tcx> {
//     let lit = |n: u128| ConstantKind::from_bits(tcx, n, ParamEnv::empty().and(tcx.types.u64));
//     let op =
//         |n: u128| Operand::Constant(Box::new(Constant { span, user_ty: None, literal: lit(n) }));
//     let rvalue = Rvalue::Use(op(0x77));
//     let new_temp = patch.new_temp(tcx.types.u64, span);
//     let place = Place::from(new_temp);
//     patch.add_assign(loc, place, rvalue);
//     place
// }

// fn patch_terminator_with_asm<'tcx>(
//     tcx: TyCtxt<'tcx>,
//     patch: &mut MirPatch<'tcx>,
//     array_raw_ptr_place: Place<'tcx>,
//     destination: BasicBlock,
//     block_to_patch: BasicBlock,
//     zzq_result: Place<'tcx>,
// ) {
//     let template_piece = InlineAsmTemplatePiece::String(String::from(
//         "rol rdi, 3\n\
//         rol rdi, 13\n\
//         rol rdi, 61\n\
//         rol rdi, 51\n\
//         xchg rbx, rbx",
//     ));
//     //let template = std::slice::from_ref(tcx.arena.alloc(template_piece));
//     let template = tcx.arena.alloc_from_iter([template_piece]);
//     let operand1 = InlineAsmOperand::InOut {
//         reg: InlineAsmRegOrRegClass::Reg(InlineAsmReg::X86(X86InlineAsmReg::di)),
//         late: false,
//         in_value: Operand::Copy(zzq_result),
//         out_place: Some(zzq_result),
//     };
//     let operand2 = InlineAsmOperand::In {
//         reg: InlineAsmRegOrRegClass::Reg(InlineAsmReg::X86(X86InlineAsmReg::ax)),
//         value: Operand::Move(array_raw_ptr_place),
//     };

//     let asm_terminator_kind = TerminatorKind::InlineAsm {
//         template,
//         operands: vec![operand1, operand2], //Vec<InlineAsmOperand<'tcx>>,
//         options: InlineAsmOptions::empty(),
//         line_spans: &[],
//         destination: Some(destination),
//         cleanup: None,
//     };

//     patch.patch_terminator(block_to_patch, asm_terminator_kind);
// }
