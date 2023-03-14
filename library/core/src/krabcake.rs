use core::{arch::asm, ptr};

const fn vg_userreq_tool_base(a: u32, b: u32) -> u32 {
    ((a) & 0xff) << 24 | ((b) & 0xff) << 16
}

/// blah
#[derive(Debug)]
#[repr(u32)]
pub enum VgKrabcakeClientRequest {
    /// blah
    BorrowMut = vg_userreq_tool_base('K' as u32, 'C' as u32),
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
    KrabcakeRecordOverlapError = vg_userreq_tool_base('K' as u32, 'C' as u32) + 256,
}

/*
#[derive(Clone, Copy)]
#[repr(C)]
struct Data<T> {
    request_code: u64,
    arg1: *mut T,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
}*/

/// blah
#[macro_export]
macro_rules! valgrind_do_client_request_expr {
    ( $zzq_default:expr, $request_code:expr,
      $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr, $arg5:expr ) => {
        {
            /*
            let zzq_args = Data {
                request_code: $request_code as u64,
                arg1: $arg1,
                arg2: $arg2,
                arg3: $arg3,
                arg4: $arg4,
                arg5: $arg5,
            };
            */
            let zzq_args = [
                 $request_code as u64,
                 $arg1 as u64,
                 $arg2,
                 $arg3,
                 $arg4,
                 $arg5,
            ];
            let mut zzq_result = $zzq_default;
            // SAFETY:
            // This is a valid safety comment
            unsafe {
                asm!(
                    "rol rdi, 3",
                    "rol rdi, 13",
                    "rol rdi, 61",
                    "rol rdi, 51",
                    "xchg rbx, rbx",
                    inout("di") zzq_result,
                    in("ax") &zzq_args,
                );
            }
            zzq_result
        }
    }
}

/// blah
#[macro_export]
macro_rules! kc_borrow_mut {
    ( $data:expr ) => {{
        let place = ptr::addr_of_mut!($data);
        valgrind_do_client_request_expr!(
            place,
            VgKrabcakeClientRequest::BorrowMut,
            place,
            0x98,
            0x92,
            0x93,
            0x94
        )
    }};
}

#[doc(hidden)]
#[unstable(feature = "gen_future", issue = "50547")]
#[inline]
#[cfg_attr(not(bootstrap), lang = "insert_krabcake_request")]
pub fn insert_krabcake_request() {
    let mut val: u8 = 1;
    kc_borrow_mut!(val);
    //panic!("I HOPE THIS WORKS");
}
