use crate::MirPass;

use rustc_ast::ast::{InlineAsmOptions, InlineAsmTemplatePiece};
use rustc_middle::mir::{BasicBlock, BasicBlockData, Body, TerminatorKind};
use rustc_middle::ty::TyCtxt;

pub struct ValgrindClientRequest;

impl<'tcx> MirPass<'tcx> for ValgrindClientRequest {
    #[instrument(skip(self, tcx, body))]
    fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
        if !tcx.sess.opts.unstable_opts.instrument_krabcake {
            info!("Not instrumenting for krabcake");
            return;
        }
        info!("Instrumenting for krabcake now...");
        let bbs = body.basic_blocks_mut();

        let template_piece = InlineAsmTemplatePiece::String(String::from("nop"));
        //let template = std::slice::from_ref(tcx.arena.alloc(template_piece));
        let template = tcx.arena.alloc_from_iter([template_piece]);

        // Doesn't really work right now cause template = &mut &[InlineAsmTemplatePiece; 1]
        // and what I need is &'tcx [InlineAsmTemplatePiece]

        let asm_terminator_kind = TerminatorKind::InlineAsm {
            template,
            operands: vec![], //Vec<InlineAsmOperand<'tcx>>,
            options: InlineAsmOptions::empty(),
            line_spans: &[],
            destination: Some(bbs.next_index()),
            cleanup: None,
        };

        // For some context, I am trying to basically insert some inline assembly at the end of
        // the list of basic blocks
        // basically
        // A -> B -> C -> D -> E
        // A -> B -> C -> D -> MyNewBlockWithInlineAsm -> E

        let len = bbs.len();
        // Take last block
        let original_last_block =
            bbs.get_mut(BasicBlock::from_usize(len - 1)).expect("No last block!!");
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
        original_last_block.terminator = Some(new_terminator);

        bbs.push(new_bb);
    }
}
