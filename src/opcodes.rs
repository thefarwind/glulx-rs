// nop
pub fn op_add(l1: i32, l2: i32) -> i32 { l1.wrapping_add(l2) }
pub fn op_sub(l1: i32, l2: i32) -> i32 { l1.wrapping_sub(l2) }
pub fn op_mul(l1: i32, l2: i32) -> i32 { l1.wrapping_mul(l2) }
pub fn op_div(l1: i32, l2: i32) -> i32 { l1.wrapping_div(l2) }
pub fn op_mod(l1: i32, l2: i32) -> i32 { l1.wrapping_rem(l2) }
pub fn op_neg(l1: i32) -> i32 { -l1 }
pub fn op_bitand(l1: i32, l2: i32) -> i32 { l1 & l2 }
pub fn op_bitor(l1: i32, l2: i32) -> i32 { l1 | l2 }
pub fn op_bitxor(l1: i32, l2: i32) -> i32 { l1 ^ l2 }
pub fn op_bitnot(l1: i32) -> i32 { !l1 }
pub fn op_shiftl(l1: i32, l2: i32) -> i32 { l1 << l2 }
pub fn op_sshiftr(l1: i32, l2: i32) -> i32 { l1 >> l2 }
pub fn op_ushiftr(l1: u32, l2: u32) -> u32 { l1 >> l2 }
// jump
pub fn op_jz(l1: i32) -> bool { l1 == 0 }
pub fn op_jnz(l1: i32) -> bool { l1 != 0 }
pub fn op_jeq(l1: i32, l2: i32) -> bool { l1 == l2 }
pub fn op_jne(l1: i32, l2: i32) -> bool { l1 != l2 }
pub fn op_jlt(l1: i32, l2: i32) -> bool { l1 < l2 }
pub fn op_jge(l1: i32, l2: i32) -> bool { l1 >= l2 }
pub fn op_jgt(l1: i32, l2: i32) -> bool { l1 > l2 }
pub fn op_jle(l1: i32, l2: i32) -> bool { l1 <= l2 }
pub fn op_jltu(l1: u32, l2: u32) -> bool { l1 < l2 }
pub fn op_jgeu(l1: u32, l2: u32) -> bool { l1 >= l2 }
pub fn op_jgtu(l1: u32, l2: u32) -> bool { l1 > l2 }
pub fn op_jleu(l1: u32, l2: u32) -> bool { l1 <= l2 }
/*
pub fn op_call
pub fn op_return
pub fn op_catch
pub fn op_throw
pub fn op_tailcall
*/
pub fn op_copy(l1: u32) -> u32 { l1 }
pub fn op_copys(l1: u16) -> u16 { l1 }
pub fn op_copyb(l1: u8) -> u8 { l1 }
pub fn op_sexs(l1: u32) -> i32 { l1 as u16 as i32 }
pub fn op_sexb(l1: u32) -> i32 { l1 as i8 as i32 }
/*
pub fn op_aload
pub fn op_aloads
pub fn op_aloadb
pub fn op_aloadbit
pub fn op_astore
pub fn op_astores
pub fn op_astoreb
pub fn op_astorebit
pub fn op_stkcount
pub fn op_stkpeek
pub fn op_stkswap
pub fn op_stkroll
pub fn op_stkcopy
pub fn op_streamchar
pub fn op_streamnum
pub fn op_streamstr
pub fn op_streamunichar
pub fn op_gestalt
pub fn op_debugtrap
pub fn op_getmemsize
pub fn op_setmemsize
pub fn op_jumpabs
pub fn op_random
pub fn op_setrandom
pub fn op_quit
pub fn op_verify
pub fn op_restart
pub fn op_save
pub fn op_restore
pub fn op_saveundo
pub fn op_restoreundo
pub fn op_protect
pub fn op_glk
pub fn op_getstringtbl
pub fn op_setstringtbl
pub fn op_getiosys
pub fn op_setiosys
pub fn op_linearsearch
pub fn op_binarysearch
pub fn op_linkedsearch
pub fn op_callf
pub fn op_callfi
pub fn op_callfii
pub fn op_callfiii
// mzero
// mcopy
pub fn op_malloc
pub fn op_mfree
pub fn op_accelfunc
pub fn op_accelparam
*/
pub fn op_numtof(l1: i32) -> f32 { l1 as f32 }
pub fn op_ftonumz(li: f32) -> i32 { li.trunc() as i32 }
pub fn op_ftonumn(li: f32) -> i32 { li.round() as i32 }
pub fn op_ceil(l1: f32) -> f32 { l1.ceil() }
pub fn op_floor(l1: f32) -> f32 { l1.floor() }
pub fn op_fadd(l1: f32, l2: f32) -> f32 { l1 + l2 }
pub fn op_fsub(l1: f32, l2: f32) -> f32 { l1 - l2 }
pub fn op_fmul(l1: f32, l2: f32) -> f32 { l1 * l2 }
pub fn op_fdiv(l1: f32, l2: f32) -> f32 { l1 / l2 }
/*
pub fn op_fmod
*/
pub fn op_sqrt(l1: f32) -> f32 { l1.sqrt() }
pub fn op_exp(l1: f32) -> f32 { l1.exp() }
pub fn op_log(l1: f32) -> f32 { l1.ln() }
pub fn op_pow(l1: f32, l2: f32) -> f32 { l1.powf(l2) }
pub fn op_sin(l1: f32) -> f32 { l1.sin() }
pub fn op_cos(l1: f32) -> f32 { l1.cos() }
pub fn op_tan(l1: f32) -> f32 { l1.tan() }
pub fn op_asin(l1: f32) -> f32 { l1.asin() }
pub fn op_acos(l1: f32) -> f32 { l1.acos() }
pub fn op_atan(l1: f32) -> f32 { l1.atan() }
pub fn op_atan2(l1: f32, l2: f32) -> f32 { l1.atan2(l2) }
pub fn op_jfeq(l1: f32, l2: f32, l3: f32) -> bool {
    let l3 = l3.abs();
    (l1 + l3 > l2) && (l2 > l1 - l3)
}
pub fn op_jfne(l1: f32, l2: f32, l3: f32) -> bool {
    !op_jfeq(l1, l2, l3)
}
pub fn op_jflt(l1: f32, l2: f32) -> bool { l1 < l2 }
pub fn op_jfle(l1: f32, l2: f32) -> bool { l1 <= l2 }
pub fn op_jfgt(l1: f32, l2: f32) -> bool { l1 < l2 }
pub fn op_jfge(l1: f32, l2: f32) -> bool { l1 >= l2 }
pub fn op_jisnan(l1: f32) -> bool { l1.is_nan() }
pub fn op_jisinf(l1: f32) -> bool { l1.is_infinite() }
