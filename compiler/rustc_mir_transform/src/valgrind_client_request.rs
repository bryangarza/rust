use crate::MirPass;

use rustc_ast::ast::{InlineAsmOptions, InlineAsmTemplatePiece};
use rustc_middle::mir::{
    AggregateKind, BasicBlock, BasicBlockData, Body, Constant, ConstantKind, Operand, Place,
    Rvalue, /*InlineAsmOperand,*/ TerminatorKind,
};
use rustc_middle::ty::TyCtxt;
//use rustc_target::asm::{InlineAsmReg, InlineAsmRegClass, X86InlineAsmReg};
use rustc_middle::mir::patch::MirPatch;
use rustc_middle::mir::Location;
use rustc_middle::ty::ParamEnv;

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

        let template_piece = InlineAsmTemplatePiece::String(String::from(
            "rol rdi, 3\n\
                        rol rdi, 13\n\
                        rol rdi, 61\n\
                        rol rdi, 51\n\
                        xchg rbx, rbx",
        ));
        //let template = std::slice::from_ref(tcx.arena.alloc(template_piece));
        let template = tcx.arena.alloc_from_iter([template_piece]);

        // Doesn't really work right now cause template = &mut &[InlineAsmTemplatePiece; 1]
        // and what I need is &'tcx [InlineAsmTemplatePiece]

        /*
        let in_operand = InlineAsmOperand::In {
            reg: InlineAsmReg::Reg(InlineAsmRegClass::X86(X86InlineAsmReg::ax)),
            value: Operand::Move(Place),
        };
        */
        let asm_terminator_kind = TerminatorKind::InlineAsm {
            template,
            operands: vec![], //Vec<InlineAsmOperand<'tcx>>,
            options: InlineAsmOptions::empty(),
            line_spans: &[],
            destination: Some(bbs.next_index()),
            cleanup: None,
        };

        // For some context, I am trying to basically insert some inline assembly at the end of
        // the list of basic blockg
        // basically
        // (A -> (B -> (C -> (D -> (E -> end)))))
        // (A -> (B -> (C -> (D -> (E+asm -> (F -> end))))))

        let len = bbs.len();
        let last_block_index = BasicBlock::from_usize(len - 1);
        let statements_len = bbs[last_block_index].statements.len();
        let last_block = bbs.get(last_block_index).expect("No last block!!");

        let array_ty = tcx.mk_array(tcx.types.u64, 6);

        let lit = |n: u128| ConstantKind::from_bits(tcx, n, ParamEnv::empty().and(tcx.types.u64));
        let op = |n: u128| {
            Operand::Constant(Box::new(Constant {
                span: last_block.terminator().source_info.span,
                user_ty: None,
                literal: lit(n),
            }))
        };

        let rvalue = Rvalue::Aggregate(
            Box::new(AggregateKind::Array(tcx.types.u64)),
            vec![op(1), op(2), op(3), op(4), op(5), op(6)],
        );

        let mut patch = MirPatch::new(body);
        let new_temp = patch.new_temp(array_ty, last_block.terminator().source_info.span);
        let last_block_end = Location { block: last_block_index, statement_index: statements_len };
        patch.add_assign(last_block_end, Place::from(new_temp), rvalue);

        let new_bb = BasicBlockData {
            statements: vec![],
            terminator: last_block.terminator.to_owned(),
            is_cleanup: false,
        };

        patch.patch_terminator(last_block_index, asm_terminator_kind);
        patch.new_block(new_bb);
        patch.apply(body);
    }
}
