use core::arch::asm;

mod vg_c_compatible {
    const fn vg_userreq_tool_base(a: u64, b: u64) -> u64 {
        ((a) & 0xff) << 24 | ((b) & 0xff) << 16
    }

    /// blah
    #[allow(dead_code)] // For currently unused variants
    #[derive(Debug, Clone, Copy)]
    #[repr(u64)]
    pub(super) enum ValgrindClientRequestCode {
        /// blah
        BorrowMut = vg_userreq_tool_base('K' as u64, 'C' as u64),
        /// blah
        BorrowShr,
        /// blah
        AsRaw,
        /// blah
        AsBorrowMut,
        /// blah
        AsBorrowShr,
        /// blah
        RetagFnPrologue,
        /// blah
        RetagAssign,
        /// blah
        RetagRaw,
        /// blah
        IntrinsicsAssume,
        /// blah
        KrabcakeRecordOverlapError = vg_userreq_tool_base('K' as u64, 'C' as u64) + 256,
    }

    /// Ultimately, this should map to a [u64; 6], which is what Valgrind expects
    #[derive(Debug, Clone, Copy)]
    #[repr(C)]
    struct ValgrindClientRequest<T>
    where
        T: Into<u64>,
    {
        req_code: ValgrindClientRequestCode,
        args: [T; 5],
    }
}

/// blah
/// #[doc(hidden)]
#[unstable(feature = "gen_future", issue = "50547")]
#[cfg_attr(not(bootstrap), lang = "KrabcakeRequest")]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum KrabcakeRequest {
    /// blah
    IntrinsicsAssume {
        /// blah
        flag: bool,
    },
}

#[doc(hidden)]
#[unstable(feature = "gen_future", issue = "50547")]
#[inline(never)]
#[cfg_attr(not(bootstrap), lang = "insert_krabcake_request")]
pub fn insert_krabcake_request(req: KrabcakeRequest) {
    // `res` not used for IntrinsicsAssume request
    let mut res = false as u64;
    let args: [u64; 6] = match req {
        KrabcakeRequest::IntrinsicsAssume { flag } => [
            vg_c_compatible::ValgrindClientRequestCode::IntrinsicsAssume as u64,
            flag.into(),
            0,
            0,
            0,
            0,
        ],
    };

    // SAFETY:
    // This is a valid safety comment
    unsafe {
        asm!(
            "rol rdi, 3",
            "rol rdi, 13",
            "rol rdi, 61",
            "rol rdi, 51",
            "xchg rbx, rbx",
            inout("di") res,
            in("ax") &args,
        );
    }
    let _ = res;
    //panic!("I HOPE THIS WORKS");
}
