/// Data load location information for an opcode
pub enum Load {

    /// Use contained constant as argument.
    Const(i32),

    /// Load argument from the contained memory address.
    Addr(u32),

    /// Pop argument off the stack.
    Pop,

    /// Load argument from the contained stack frame address.
    Frame(u32),

    /// Load argument from the contained RAM address.
    Ram(u32),
}


/// Data save location information for an opcode
pub enum Save {

    /// Discard result.
    Null,

    /// Save result at the contained memory address.
    Addr(u32),

    /// Push result onto the stack.
    Push,

    /// Save result at the contained stack frame address.
    Frame(u32),

    /// Save result at the contained RAM address.
    Ram(u32),
}


/// Glulx opcodes
pub enum Opcode {

    /// Does nothing.
    NOP,

    /// Add together two `i32`.
    ADD(Load, Load, Save),

    /// Substract second `i32` from first `i32`.
    SUB(Load, Load, Save),

    /// Multiply together two `i32`.
    MUL(Load, Load, Save),

    /// Divide first `i32` by a second `i32`.
    DIV(Load, Load, Save),

    /// Mod together two `i32`.
    MOD(Load, Load, Save),

    /// Negate an `i32`.
    NEG(Load, Save),

    /// Bitwise AND together two `i32`.
    BITAND(Load, Load, Save),

    /// Bitwise OR together two `i32`.
    BITOR(Load, Load, Save),

    /// Bitwise XOR together two `i32`.
    BITXOR(Load, Load, Save),

    /// Bitwise NOT an `i32`.
    BITNOT(Load, Save),
    SHIFTL(Load, Load, Save),
    SSHIFTR(Load, Load, Save),
    USHIFTR(Load, Load, Save),

    /// Jump to a memory address.
    JUMP(Load),

    /// Jump if the first `i32` is zero.
    JZ(Load, Load),

    /// Jump if the first `i32` is not zero.
    JNZ(Load, Load),

    /// Jump if the first two `i32`s are equal.
    JEQ(Load, Load, Load),

    /// Jump if the first two `i32`s are not equal.
    JNE(Load, Load, Load),

    /// Jump if the first `i32` is less than the second `i32`.
    JLT(Load, Load, Load),

    /// Jump if the first `i32` is greater than or equal to the second `i32`.
    JGE(Load, Load, Load),

    /// Jump if the first `i32` is greater than to the second `i32`.
    JGT(Load, Load, Load),

    /// Jump if the first `i32` is less than or equal to the second `i32`.
    JLE(Load, Load, Load),

    /// Jump if the first `u32` is less than the second `u32`.
    JLTU(Load, Load, Load),

    /// Jump if the first `u32` is greater than or equal to the second `u32`.
    JGEU(Load, Load, Load),

    /// Jump if the first `u32` is greater than to the second `u32`.
    JGTU(Load, Load, Load),

    /// Jump if the first `u32` is less than or equal to the second `u32`.
    JLEU(Load, Load, Load),
    CALL(Load, Load, Save),
    RETURN(Load),
    CATCH(Save, Load),
    THROW(Load, Load),
    TAILCALL(Load, Load),
    COPY(Load, Save),
    COPYS(Load, Save),
    COPYB(Load, Save),
    SEXS(Load, Save),
    SEXB(Load, Save),
    ALOAD(Load, Load, Save),
    ALOADS(Load, Load, Save),
    ALOADB(Load, Load, Save),
    ALOADBIT(Load, Load, Save),
    ASTORE(Load, Load, Load),
    ASTORES(Load, Load, Load),
    ASTOREB(Load, Load, Load),
    ASTOREBIT(Load, Load, Load),
    STKCOUNT(Save),
    STKPEEK(Load, Save),
    STKSWAP,
    STKROLL(Load, Load),
    STKCOPY(Load),
    STREAMCHAR(Load),
    STREAMNUM(Load),
    STREAMSTR(Load),
    STREAMUNICHAR(Load),
    GESTALT(Load, Load, Save),
    DEBUGTRAP(Load),
    GETMEMSIZE(Save),
    SETMEMSIZE(Load, Save),
    JUMPABS(Load, Load, Load),
    RANDOM(Load, Save),
    SETRANDOM(Save),
    QUIT,
    VERIFY(Save),
    RESTART,
    SAVE(Load, Save),
    RESTORE(Load, Save),
    SAVEUNDO(Save),
    RESTOREUNDO(Save),
    PROTECT(Load, Load),
    GLK(Load, Load, Save),
    GETSTRINGTBL(Save),
    SETSTRINGTBL(Load),
    GETIOSYS(Save, Save),
    SETIOSYS(Load, Load),
    LINEARSEARCH(Load, Load, Load, Load, Load, Load, Load, Save),
    BINARYSEARCH(Load, Load, Load, Load, Load, Load, Load, Save),
    LINKEDSEARCH(Load, Load, Load, Load, Load, Load, Save),
    CALLF(Load, Save),
    CALLFI(Load, Load, Save),
    CALLFII(Load, Load, Load, Save),
    CALLFIII(Load, Load, Load, Load, Save),
    MZERO(Load, Load),
    MCOPY(Load, Load, Load),
    MALLOC(Load, Save),
    MFREE(Load),
    ACCELFUNC(Load, Load),
    ACCELPARAM(Load, Load),

    /// Convert `i32` to `f32`.
    NUMTOF(Load, Save),

    /// Truncate `f32` to `i32`.
    FTONUMZ(Load, Save),

    /// Round `f32` to `i32`.
    FTONUMN(Load, Save),

    /// Ceil of `f32` as `f32`.
    CEIL(Load, Save),

    /// floor of `f32` as `f32`.
    FLOOR(Load, Save),

    /// Add together two `f32`s.
    FADD(Load, Load, Save),

    /// Subtract the second `f32` from the first `f32`.
    FSUB(Load, Load, Save),

    /// Multiply together two `f32`s.
    FMUL(Load, Load, Save),

    /// Divide the first `f32` by the second `f32`.
    FDIV(Load, Load, Save),
    FMOD(Load, Load, Save, Save),

    /// Return square root of `f32`.
    SQRT(Load, Save),

    /// Return natural exponent of `f32`.
    EXP(Load, Save),

    /// Return natural log of `f32`.
    LOG(Load, Save),

    /// Return power of first `f32` to second `f32`.
    POW(Load, Load, Save),

    /// Return sin of `f32`.
    SIN(Load, Save),

    /// Return cos of `f32`.
    COS(Load, Save),

    /// Return tan of `f32`.
    TAN(Load, Save),

    /// Return arcsin of `f32`.
    ASIN(Load, Save),

    /// Return arccos of `f32`.
    ACOS(Load, Save),

    /// Return arctan of `f32`.
    ATAN(Load, Save),
    ATAN2(Load, Load, Save),
    JFEQ(Load, Load, Load, Load),
    JFNE(Load, Load, Load, Load),
    JFLT(Load, Load, Load),
    JFLE(Load, Load, Load),
    JFGT(Load, Load, Load),
    JFGE(Load, Load, Load),

    /// Jump if `f32` is `f32::NAN`.
    JISNAN(Load, Load),

    /// Jump if `f32` is `f32::INFINITY`.
    JISINF(Load, Load),
}
