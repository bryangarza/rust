use crate::MirPass;

use rustc_ast::ast::{InlineAsmOptions, InlineAsmTemplatePiece};
use rustc_middle::mir::{BasicBlock, BasicBlockData, Body, TerminatorKind, /*InlineAsmOperand,*/ Statement, StatementKind, Rvalue, AggregateKind, Operand, Constant, ConstantKind, Place};
use rustc_middle::ty::TyCtxt;
use rustc_index::vec::Idx;
use rustc_middle::mir::Local;
//use rustc_target::asm::{InlineAsmReg, InlineAsmRegClass, X86InlineAsmReg};
use rustc_middle::ty::ParamEnv;
use rustc_middle::mir::LocalDecl;
use core::iter::Extend;

pub struct ValgrindClientRequest;

impl<'tcx> MirPass<'tcx> for ValgrindClientRequest {
    #[instrument(skip(self, tcx, body))]
    fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
        if !tcx.sess.opts.unstable_opts.instrument_krabcake {
            info!("Not instrumenting for krabcake");
            return;
        }
        info!("Instrumenting for krabcake now...");
        let next_local = body.local_decls.len();
        let bbs = body.basic_blocks_mut();

        let template_piece = InlineAsmTemplatePiece::String(
            String::from("rol rdi, 3\n\
                        rol rdi, 13\n\
                        rol rdi, 61\n\
                        rol rdi, 51\n\
                        xchg rbx, rbx"));
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
        // Take last block
        let original_last_block =
            bbs.get_mut(BasicBlock::from_usize(len - 1)).expect("No last block!!");

        /*
        let original_last_block_last_stmt_source_info = original_last_block.statements
            .last()
            .expect("Last block has no statements!")
            .source_info
            .clone();
            */

        let lit = |n: u128| ConstantKind::from_bits(tcx, n, ParamEnv::empty().and(tcx.types.u64));
        let op = |n: u128| {
            Operand::Constant(Box::new(Constant {
                span: original_last_block.terminator().source_info.span,
                user_ty: None,
                literal: lit(n),
            }))
        };

        let array_ty = tcx.mk_array(tcx.types.u64, 6);
        let rvalue = Rvalue::Aggregate(
            Box::new(AggregateKind::Array(tcx.types.u64)),
            vec![op(1), op(2), op(3), op(4), op(5), op(6)],
        );

        /*
        let place = Place {
            local: Local::from_usize(1),
            projection: List::empty(),
        };
        */
        let local_decl = LocalDecl::new(array_ty, original_last_block.terminator().source_info.span);
        let place = Place::from(Local::new(next_local));
        let kind = StatementKind::Assign(Box::new((place, rvalue)));

        let stmt1 = Statement {
            source_info: original_last_block.terminator().source_info,
            kind,
        };

        // create new block whose terminator is clone of original
        let new_bb = BasicBlockData {
            statements: vec![],
            terminator: original_last_block.terminator.to_owned(),
            is_cleanup: false,
        };

        // Duplicate the last block's terminator
        let mut new_terminator =
            original_last_block.terminator.as_ref().expect("No terminator!!").clone();

        // modify original terminator (should now be asm pointing to next block)
        new_terminator.kind = asm_terminator_kind;
        original_last_block.statements.push(stmt1);
        original_last_block.terminator = Some(new_terminator);

        bbs.push(new_bb);
        body.local_decls.extend(vec![local_decl]);
    }
}
