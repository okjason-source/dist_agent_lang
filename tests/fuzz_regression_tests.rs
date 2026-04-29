//! Fuzz regression tests
//!
//! These tests ensure that previously-discovered crash inputs no longer cause panics.
//! Inputs are from fuzz/artifacts/ (parser crashes from deeply nested input).
//!
//! Run with: cargo test --test fuzz_regression_tests

use dist_agent_lang::lexer::Lexer;
use dist_agent_lang::parser::Parser;
use dist_agent_lang::runtime::Runtime;

/// Parser crash: deeply nested Y(YY(YYY... pattern caused stack overflow.
/// Fixed by adding MAX_RECURSION_DEPTH limit (25) in parser.
const CRASH_PARSER_DEEP_NESTING_Y: &str = "Y(YY(A(((YYY(YY(YYY((YYYYYY((YYY(A(((YYYY(YYY((A(YYYYYY((YYY(YYY(((YYY(YYY((YYY(A(((YYY(YYY((A(YYYYYY((YYY(YYY((YYYY((A(YYYYYY((YYW(YYY(((YYYYYYY((YYY(YYY((YYYYYY(Y((YYY(YYY((YYYYYY((YYY(YYY(YYY((YYY(A(((YYY(YYY((A(YYYYYY((YYY(YYY(((YYY(YYY((YYYYYY((YYY(YYY((A(YYYYYY((YYY(YYY((YYYY((A(YYYYYY((YYW(YYY(((YYYYYYY((YYY(YYY((YYYYYY(Y((YYY(YYY((YYYYYY((YYY(YYY(YYY((YYY(A(((YYY(YYY((A(YYYYYY((YYY(YYY(((YYY(YYY((YYYYYY((YYY(YY((YYY(A(((YYY(YYY((A(YYYYYY((YYY(YYY(((YYY(YYY((YYY((A(YYYYYY((YYY(YYY(((YYY(YYY((YYY(A(((YYY(YYY((A(YYYYYY((YYY(YYY((YYYY((A(YYYYYY((YYW(YYY(((YYYYYYY((YYY(YYY((YYYYYY(Y((YYY(YYY((YYYYYY((YYY(YYY(YYY((YYY(A(((YYY(YYY((A(YYYYYY((YYY(YYY(((YYY(YYY((YYYYYY((YYY(YYY((A(YYYYYY((YYY(YYY((YYYY((A(YYYYYY((YYW(YYY(((YYYYYYY((YYY(YYY((YYYYYY(Y((YYY(YYY((YYYYYY((YYY(YYY(YYY((YYY(A(((YYY(YYY((A(YYYYYY((YYY(YYY(((YYY(YYY((YYYYYY((YYY(YY((YYY(A(((YYY(YYY((A(YYYYYY((YYY(YYY(((YYY(YYY((YYY(A(((YYY(YYY((A(YYYYYY((YYY(YYY((YYYYY((YYY(YYY((YYY(A(((YYY(YYY((A(YYYYYY((YYY(YYY(((YYY(YYY((YYY(A(((YYY(YYY((A(YYYYYY((YYY(YY=z=KzAK= LYYY(YYY((YYYYYY((YYY(YYY((YYY(A(((YYY(YYY((A(YYYYYY((YYY(YYY(((YYY(YYY((YYY(A(((YYY(YYY((A(YYYYYY((YYY(YY(A(((YYY(YY(YYY((YYYYYY((YYY(A(((YYYY(YYY((A(YYYYYY((YYY(YYY(((YYY(YYY((YYY(A(((YYY(YYY((A(YYYYYY((YYY(YYY((YYYY((A(YYYYYY((YYW(YYY(((YYYYYYY((YYY(YYY((YYYYYY(Y((YYY(YYY((YYYYYY((YYY(YYY(YYY((YYY(A(((YYY(YYY((A(YYYYYY((YYY(YYY(((YYY(YYY(((A(YYYYYY((YYY(YYY(((YYY(YYY((YYY(A(((YYY(YYY((A(YYYYYY((YYY(YYY((YYYY((A(YYYYY(YYY(A(((YYY(YYY((A(YYYYYY((YYY(YYY((YYYYY((YYY(YYY((YYY(A(((YYY(YYY((A(YYYYYY((YYY(YYY(((YYY(YYY((YYY(A(((YYY(YYY((A(YYYYYY((YYY(YY=z=KzAK= LYYY(YYY((YYYYYY((YYY(YYY((YYY(A(((YYY(YYY((A(YYYYYY((YYY(YYY(((YYY(YYY((YYY(A(((YYY(YYY((A(YYYYYY((YYY(YY(A(((YYY(YY(YYY((YYYYYY((YYY(A(((YYYY(YYY((A(YYYYYY((YYY(YYY(((YYY(YYY((YYY(A(((YYY(YYY((A(YYYYYY((YYY(YYY((YYYY((A(YYYYYY((YYW(YYY(((YYYYYYY((YYY(YYY((YYYYYY(Y((YYY(YYY((YYYYYY((YYY(YYY(YYY((YYY(A(((YYY(YYY((A(YYYYYY((YYY(YYY(((YYY(YYY(((A(YYYYYY((YYY(YYY(((YYY(YYY((YYY(A(((YYY(YYY((A(YYYYYY((YYY(YYY((YYYY((A(YYYYYY((YYW(YYY(((YYYYYYY((YYY(YYY((YYYYYY(Y((YYY(YYY((YYYYYY((YYY(YYY(YYY((YYY(A(((YYY(YYY((A(YYYYYY((YYY(YYY(((YYY(YYY((YYYYYY((YYY(YYY((YYY(A(((YYY(YYY((A(YYYYYY((YYY(YYY(((YYY(YYY((Yz=Kz=K= L=LK=KL==z L=!!L L=LK=K= L=z=Kz=K= L=L L=L=!!L!";

/// Parser crash: deeply nested {{{}R pattern caused stack overflow.
/// Fixed by adding MAX_RECURSION_DEPTH limit in parser.
const CRASH_PARSER_DEEP_NESTING_BRACES: &str = "{{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{R{{R{{{}}R{{{}{}R{{{}}R{{{}R{R{{{R{{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}RR{{{}{R{{{}}R{{{}R{R{{RR{{}}}}}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{R{{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{R{{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{A{{R{{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{R{{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{R{{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{RR{{}}}}}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{R{{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{R{{R{{{}}R{{{}R{R{{RR{{}{}}R}R{{{}}{{}}R{{{}R{R{{{R{{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{A{{R{{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{R{{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{R{{R{{{}}R{{{}R{R{{RR{{}{}}R}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{R{{R{{{}}R{{{}{}R{{{}}R{{{}R{R{{{R{{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{RR{{}}}}}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{R{{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{R{{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{A{{R{{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{R{{{{R{{{}R{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{R{{R{{{}}R{{{}{}R{{{}}R{{{}R{R{{{R{{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{RR{{}}}}}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{R{{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{R{{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{A{{R{{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{R{{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{R{{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{RR{{}}}}}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{R{{{{}R{{{}}R{{{}{}{}}R{{{}R{{{}}R{{{}R{R{{{R{{R{{{R{{{R{{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}RR{{{}{R{{{}}R{{{}R{R{{RR{{}}}}}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{R{{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{R{{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{A{{R{{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{R{{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{R{{R{{{}}R{{{}{}R{{{}}R{{{}R{R{{{R{{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{RR{{}}}}}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{R{{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{R{{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{A{{R{{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{R{{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{R{{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{RR{{}}}}}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{R{{{{}R{{{}}R{{{}{}{}}R{{{}R{{{}}R{{{}R{R{{{R{{R{{{}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{}}{}}Rt{{}R{R{{RR}}}}R{{}}}}}{}}Rt{{}R{R{{{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{RR{{}}}}}{}}Rt{{}R{R{{RR}}}}R{{}}}}}{}}Rt{{}R}}R{{{}R{R{{RR{{}{}}R{{{}R{{{}}R{{{}R{R{{{}}{}}Rt{{}R{R{{RR}}}}R{{}}}}}{}}Rt{{}R{R{{{{{}R{{{}}R{{{}R{R{{{}}R{{{}R{R{{RR{{}}}}}{}}Rt{{}R{R{{RR}}}}R{{}}}}}{}}Rt{{}R{R{{RR}}}}";

/// Runtime OOM: oom-38d1493ab2f9b621702c14b508d9640c3d409e22
/// Input '-0..108281082m1' parses as range 0..108281082, causing ~7.8GB allocation.
/// Fixed by adding MAX_RANGE_LEN (100_000) in runtime engine.
const OOM_RUNTIME_RANGE: &str = "-0..108281082m1";

/// Runtime OOM: oom-3622727b78e58d2dc1d3c91ac2d53844ab66bbc3
/// Input '-0..810820811m2m1' - same pattern, larger range. Fixed by MAX_RANGE_LEN.
const OOM_RUNTIME_RANGE_3622727B: &str = "-0..810820811m2m1";

/// Parser timeout: timeout-065e9bfdbe4c846b00bd7e0ee3c019be984e61c6
/// Original artifact was "deeply nested A.A.A.A/A(l[h[CT=..." - not in repo (fuzz/artifacts is gitignored).
/// Inline equivalent: deeply nested call pattern so MAX_RECURSION_DEPTH (25) applies and we don't hang.
const TIMEOUT_PARSER_DEEP_NESTING: &str = "A(A(A(A(A(A(A(A(A(A(A(A(A(A(A(A(A(A(A(A(A(A(A(A(A(A(0";

/// Runtime slow unit: slow-unit-f872bd43ba9128449c4b110a982134a43f03ca59
const SLOW_UNIT_RUNTIME_F872BD43: &str = "D=B=D=B=D=B=D=B=\tki[D=B=DT2-TrT22--2-TrT/-2-2-2T-2jT2-TrT-2-2-2T-2jT*-2-2j2-2jTjr2-T-jr2-2-0jT2-2jr2-T-2TTrT-2-2-2T-2jT2-TrT-2-2-2T-2jT2-2-2j2-2jTjr2-2-2jT2-2jr2-TT-jr2-2-2jT2-2jr2-T-jr2-T-22-2-2-2T-2jT2-2j2T-2j-T2-TrT-2-2-2T-2jT2-2-2j2-2jTjr2-T-jr2-2-2jT2-2jr2-T-jr2-T-22-2-2-2T-2-2jT2-TrT-\t2-2-2T-2jT2-2-2j2-2jTjr2-T-jr2-2-2jT2-2jr2-T-jr2-T-22-2-2-2T-2jT2-2j2T-2j-2";

/// Runtime crash: crash-3a95ae7b15d7cfc2f50254b2448814f6b406ea39
/// Long chain of && with numeric/range operands caused stack overflow in evaluate_expression.
/// Fixed by adding MAX_EVAL_DEPTH (512) in runtime engine.
const CRASH_RUNTIME_STACK_OVERFLOW_AND: &str = "7.31&&7-331&&7-33.33&&7..31&&-33&&7..31&&7..3&&7.1&&7-331&&7-33&&7..31&&7..33&&7..31&&3.332&&7-333&&7..31&&71&&3.332&&7-333&&7..31&&7..33&&7..31&&3.332&&7-666&&7..31&&7..33&&7..331&&7..33&&7..31&&-33&&7..31&&7..3&&7.1&&7-331&&7-33&&7..31&&7..33&&7..31&&3.332&&7-333&&7..31&&71&&3.332&&7-333&&7..31&&7..33&&7..31&&3.332&&7-666&&7..31&&7..33&&7..331&&7..33&&7..31&&-33&&7..31&&7..33&&7..31&&3.31&&7..33&&7..31&&-33&&7..31&&7..3&&7.1&&7-331&&7-33&&7..31&&7..33&&7..31&&3.332&&7-333&&7..31&&71&&3.332&&7-333&&7..31&&7..33&&7..31&&3.332&&7-666&&7..31&&7..33&&7..331&&31&&7-33&&7..31&&8..33&&7..31&&3.332&&7-666&&7..31&&7..34&&7..331&&7..33&&7.31&&7..33&&7..31&&-33&&7..31&&7..33&&7..31&&3.332&&7-666&&7..31&&7..33&&7..331&&7..33&&7..31&&-33&&7..31&&7..33&&7..31&&3.332&&7-6663&&7..31&&-33&&7..31&&7..33&&7..31&&3.31&&7..33&&7..31&&-33&&7..31&&7..3&&7.1&&7-331&&7-33&&7..31&&7..33&&7..31&&3.332&&7-333&&7..31&&71&&3.332&&7-333&&7..31&&7..33&&7..31&&3.332&&7-666&&7..31&&7..33&&7..331&&7..33&&7..31&&-33&&7..31&&7..33&&7..31&&3.332&&7-666&&7..31&&7..33&&7..331&&31&&7-33&&7..31&&8..33&&7..31&&3.332&&7-666&&7..31&&7..34&&7..331&&7..33&&7.31&&7..33&&7..31&&-33&&7..31&&7..33&&7..31&&3.332&&7-666&&7..31&&7..33&&7..331&&31&&7-33&&7..31&&8..33&&7..31&&3.332&&7-666&&7..31&&7..34&&7..331&&7..33&&7..31&&-3&&7-33&&7..31&&7..33&&7..31&&3.332&&7-666&&7..31&&7..33&&7..331&&7..33&&7..31&&-33&&7..31&&7..33&&7..31&&3.332&&7-666&&7..31&&1&&7..33&&7..31&&3.332&&7-666&&7..31&&7..33&&7..331&&31&&7-33&&7..31&&8..33&&7..31&&3.31&&7..33&&7..31&&-33&&7..31&&7..33&&7..31&&3.332&&7-666&&7..31&&332&&7-666&&7..31&&7..33&&7..331&&31&&7-33&&7..31&&8..33&&7..31&&3.332&&7-666&&7..31&&7..34&&7..331&&7..33&&7..31&&-3&&7-33&&7..31&&7..33&&7..31&&3.332&&7-666&&7..31&&7..33&&7..331&&7..33&&7..31&&-33&&7..3&&7..31&&3.332&&7-666&&7..31&&7..33&&7..331&&7..33&&7..31&&-33&&7..31&&7..33&&7..31&&3.332&&7-666&&7..31&&7..33&&7..331&&31&&7-33&&7..31&&8..33&&7..31&&3.332&&7-666&&7..31&&7..33&&7..331&&7..33&&7..31&&-3&&7-33&&7..31&&7..33&&7..31&&3.332&&7-666&&7..31&&7..33&&7..331&&7..33&&7..31&&-33&&7..31&&7..33&&7..31&&3.332&&7-666&&7..31&&7..33&&7..331&&31&&7-33&&7..31&&8..33&&7.1&&7-331&&7-33&&7..31&&7..33&&7..31&&3.332&&7-333&&7..31&&71&&3.332&&7-333&&7..31&&7..33&&7..31&&3.332&&7-666&&7..31&&7..33&&7..331&&7..33&&7..31&&-33&&7..31&&7..33&&7..31&&3.332&&7-666&&7..31&&7..33&&7..331&&31&&7-33&&7..31&&8..33&&7..31&&3.332&&7-666&&7..31&&7..34&&7..331&&7..33&&7..31&&-3&&7-33&&7..31&&7..33&&7..31&&3.332&&7-666&&7..31&&7..33&&7..331&&7..33&&7..31&&-33&&7..31&&7..33&&7..31&&3.332&&7-666&&7..31&&1&&7..33&&7..31&&3.332&&7-666&&7..31&&7..33&&7..331&&31&&7-33&&7..31&&8..33&&7..31&&3.31&&7..33&&7..31&&-33&&7..31&&7..33&&7..31&&3.332&&7-666&&7..31&&7..33&&7.31&&3.332&&7-666&&7..31&&7..33&&7..331&&7..33&&7..31&&-33&&7..31&&7..33&&7..31&&3.332&&7-666&&7..31&&7..33&&7..331&&7-332&&7-33&&3&&7";

/// Runtime crash: crash-625b3824e15cc661a1c4b8dcf6fe2ff136c82882
/// Full pipeline (lex→parse→execute). May now fail at parse (recursion) before runtime.
const CRASH_RUNTIME_625B3824: &str = ";;L;LL;L;;LLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLL;;LH;LL;;LL;;LL;L;;LL;;LH;LL;;LL;;LL;LL;;L{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{;L;2L;L;;3{;;L;2L;L48}48}{{{{{{{{{LL;{{{{{LL;;LL{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{;L;2L;L;;3{;;L;2L;L48}48}{{{{{{{{{{{;LL;LL;;LL{{{{{{{{{{{{{{0L;LL;L;;LL;;LH;LL;;LL;;LL;L;;LL;;LH;LL;;LL;;LL;LL;;L{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{;L;1L;L;;3{;;L;2L;L48}48}{{{;L;2L;L;;3{;;L;2L;L48}48}{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{;LL;LL;;LL{{{{{{{{{{;;L;2L;L48}48}{{{{{{{{{{{{{{{{{{{{{{{{;LL;LL;;LL{{{{{{{2L;L24}48}{{{;L;2L;L2L;L;;3{;;L;2L;L48}48}{{{{{{{{{LL;{{{{{LL;;LL{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{;L;2L;L;;3{;;L;2L;L48}48}{{{{{{{{{{{;LL;LL;;LL{{{{{{{{{{{{{{0L;LL;L;;LL;;LH;LL;;LL;;LL;L;;LL;;LH;LL;;LL;;LL;LL;;L{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{L;LL;L;;LLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLL;;LH;LL;;LL;;LL;L;;LL;;LH;LL;;LL;;LL;LL;;L{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{;L;2L;L;;3{;;L;2L;L48}48}{{{{{{{{{LL;{{{{{LL;;LL{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{;L;2L;L;;3{;;L;2L;L48}48}{{{{{{{{{{{;LL;LL;;LL{{{{{{{{{{{{{{0L;LL;L;;LL;;LH;LL;;LL;;LL;L;;LL;;LH;LL;;LL;;LL;LL;;L{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{;L;1L;L;;3{;;L;2L;L48}48}{{{;L;2L;L;;3{;;L;2L;L48}48}{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{;LL;LL;;LL{{{{{{{{{{;;L;2L;L48}48}{{{{{{{{{{{{{{{{{{{{{{{{;LL;LL;;LL{{{{{{{{{{{{{{0L;LL;L;;LL;;LH;LL;;LL;;LL;L;;LL;;LH;LL;9LL;{{{{{{{{{{{{{{{{{{{{{{{y{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{;L;1L;L;;3{;;L;{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{;L;1L;L;;3{;;L;2L;L48}48}{{{;L;2L;L;;3{;;L;2L;L48}48}{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{;LL;LL;;LL{{{{{{{{{{;;L;2L;L48}48}{{{{{{{{{{{{{{{{{{{{{{{{;LL;LL;;LL{{{{{{{2L;L24}48}{{{;L;2L;L2L;L;;3{;;L;2L;L48}48}{{{{{{{{{LL;{{{{{LL;;LL{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{;L;2L;L;;3{;;L;2L;L48}48}{{{{{{{{{{{;LL;LL;;LL{{{{{{{{{{{{{{0L;LL;L;;LL;;LH;LL;;LL;;LL;L;;LL;;LH;LL;;LL;;LL;LL;;L{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{;L;1L;L;;3{;;L;2L;L48}48}{{{;L;2L;L;;3{;;L;2L;L48}48}{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{;LL;LL;;LL{{{{{{{{{{;;L;2L;L48}48}{{{{{{{{{{{{{{{{{{{{{{{{;LL;LL;;LL{{{{{{{{{{{{{{0L;LL;L;;LL;;LH;LL;;LL;;LL;L;;LL;;LH;LL;9LL;{{{{{{{{{{{{{{{{{{{{{{{y{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{;L;1L;L;;3{;;L;{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{;L;1L;;3{;;L;2L;L48}48}{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{;LL;LL;;LL{{{{{{{{{{{{{{0{{{{{{{{{{{{{{{;L;1L;L;;3{;;L;2L;L48}48}{{{;L;2L;L;;3{;;L;2L;L48}48}{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{;LL;LL;;LL{{{{{{{{{{;;L;2L;L48}48}{{{{{{{{{{{{{{{{{{{{{{{{;LL;LL;;LL{{{{{{{{{{{{{{0L;LL;L;;LL;;LH;LL;;LL;L;;LL;;LH;LL;9LL;{{{{{{{{{{{{{{{{{{{{{{{y{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{;L;1L;L;;3{;;L;{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{;L;1L;;3{;;L;2L;L48}48}{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{;LL;LL;;LL{{{{{{{{{{{{{{0{;LL{;LL;LL;;LL";

#[test]
fn test_fuzz_parser_crash_deep_nesting_y_no_panic() {
    // Regression: crash-cbaac84bd697ea3aaae793957a152987877e458e
    // Must not panic. Use 8MB stack - test default (2MB) overflows before recursion limit.
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let lexer = Lexer::new(CRASH_PARSER_DEEP_NESTING_Y);
            let tokens_with_pos = match lexer.tokenize_with_positions_immutable() {
                Ok(t) => t,
                Err(_) => return,
            };
            let mut parser = Parser::new_with_positions(tokens_with_pos);
            let _ = parser.parse();
        })
        .unwrap()
        .join()
        .unwrap();
}

/// Parser try/catch crash artifacts: deeply nested try { } c try { } ... caused stack overflow
/// because try/catch/finally did not pass depth. Fixed by passing depth through parse_try_statement.
/// Fuzz harness now uses 8 MiB stack so recursion limit triggers before overflow; fuzzing continues.
const CRASH_PARSER_TRY_ARTIFACTS: &[&str] = &[
    "fuzz/artifacts/fuzz_parser/crash-4a6efcfc0987cd7ded43c9b78bb300e7e63953c4",
    "fuzz/artifacts/fuzz_parser/crash-75ad520854a1f0522321a704c515aade7e269873",
    "fuzz/artifacts/fuzz_parser/crash-541fa01af4770f0262e5ffd79658ddef7217fd1e",
    "fuzz/artifacts/fuzz_parser/crash-161522ed7f1f2ac04d9aa2e317697ce14c287338",
    "fuzz/artifacts/fuzz_parser/crash-b22fc9faf4e213ffe3bf83783270629b51d824c5",
];

#[test]
fn test_fuzz_parser_try_crash_artifacts_no_panic() {
    for path in CRASH_PARSER_TRY_ARTIFACTS {
        let input = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => continue, // artifact not in tree (e.g. CI)
        };
        std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(move || {
                let lexer = Lexer::new(&input);
                if let Ok(tokens_with_pos) = lexer.tokenize_with_positions_immutable() {
                    let mut parser = Parser::new_with_positions(tokens_with_pos);
                    let _ = parser.parse(); // must not panic; returns Err(recursion) or other
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }
}

#[test]
fn test_fuzz_parser_crash_deep_nesting_braces_no_panic() {
    // Regression: crash-cce88937ec81990273e1024f5b0fe147325c8f15
    // Must not panic. Use 8MB stack - test default (2MB) overflows before recursion limit.
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let lexer = Lexer::new(CRASH_PARSER_DEEP_NESTING_BRACES);
            let tokens_with_pos = match lexer.tokenize_with_positions_immutable() {
                Ok(t) => t,
                Err(_) => return,
            };
            let mut parser = Parser::new_with_positions(tokens_with_pos);
            let _ = parser.parse();
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn test_fuzz_parser_crash_deep_nesting_returns_recursion_error() {
    // Verify we get a proper recursion depth error (not silent failure).
    // Run in thread with larger stack: test default (2MB) can overflow before limit kicks in.
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let lexer = Lexer::new(CRASH_PARSER_DEEP_NESTING_Y);
            let tokens_with_pos = lexer.tokenize_with_positions_immutable().unwrap();
            let mut parser = Parser::new_with_positions(tokens_with_pos);
            let result = parser.parse();
            assert!(result.is_err(), "Deep nesting should produce parse error");
            let err_msg = format!("{:?}", result.unwrap_err());
            assert!(
                err_msg.contains("recursion") || err_msg.contains("depth"),
                "Expected recursion/depth error, got: {}",
                err_msg
            );
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn test_fuzz_runtime_oom_range_no_panic() {
    // Regression: oom-38d1493ab2f9b621702c14b508d9640c3d409e22
    // Range -0..108281082m1 caused OOM. Must not panic - runtime should return error instead.
    let lexer = Lexer::new(OOM_RUNTIME_RANGE);
    if let Ok(tokens_with_pos) = lexer.tokenize_with_positions_immutable() {
        let mut parser = Parser::new_with_positions(tokens_with_pos);
        if let Ok(program) = parser.parse() {
            let mut runtime = Runtime::new();
            let _ = runtime.execute_program(program, None);
            // Must not panic; OOM fix returns Err("Range too large") instead of allocating
        }
    }
}

#[test]
fn test_fuzz_runtime_oom_range_3622727b_no_panic() {
    // Regression: oom-3622727b78e58d2dc1d3c91ac2d53844ab66bbc3
    // Range -0..810820811m2m1. Same MAX_RANGE_LEN fix applies.
    let lexer = Lexer::new(OOM_RUNTIME_RANGE_3622727B);
    if let Ok(tokens_with_pos) = lexer.tokenize_with_positions_immutable() {
        let mut parser = Parser::new_with_positions(tokens_with_pos);
        if let Ok(program) = parser.parse() {
            let mut runtime = Runtime::new();
            let _ = runtime.execute_program(program, None);
        }
    }
}

#[test]
fn test_fuzz_parser_timeout_deep_nesting_no_panic() {
    // Regression: timeout-065e9bfdbe4c846b00bd7e0ee3c019be984e61c6
    // Deeply nested A.A.A.A/A(l[h[CT=... pattern. Must not panic/timeout - MAX_RECURSION_DEPTH applies.
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let lexer = Lexer::new(TIMEOUT_PARSER_DEEP_NESTING);
            if let Ok(tokens_with_pos) = lexer.tokenize_with_positions_immutable() {
                let mut parser = Parser::new_with_positions(tokens_with_pos);
                let _ = parser.parse();
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn test_fuzz_runtime_slow_unit_no_panic() {
    // Regression: slow-unit-f872bd43ba9128449c4b110a982134a43f03ca59
    // Repetitive D=B=... pattern. Must complete without panic; short-circuit && and timeout help.
    let lexer = Lexer::new(SLOW_UNIT_RUNTIME_F872BD43);
    if let Ok(tokens_with_pos) = lexer.tokenize_with_positions_immutable() {
        let mut parser = Parser::new_with_positions(tokens_with_pos);
        if let Ok(program) = parser.parse() {
            let mut runtime = Runtime::new();
            let _ = runtime.execute_program(program, None);
        }
    }
}

#[test]
fn test_fuzz_runtime_crash_625b3824_no_panic() {
    // Regression: crash-625b3824e15cc661a1c4b8dcf6fe2ff136c82882
    // Full pipeline: lex → parse → execute. Must not panic.
    // May now fail at parse (recursion limit) before reaching runtime.
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let lexer = Lexer::new(CRASH_RUNTIME_625B3824);
            if let Ok(tokens_with_pos) = lexer.tokenize_with_positions_immutable() {
                let mut parser = Parser::new_with_positions(tokens_with_pos);
                if let Ok(program) = parser.parse() {
                    let mut runtime = Runtime::new();
                    let _ = runtime.execute_program(program, None);
                }
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

/// Runtime timeout: timeout-acecfa5bd1abe812a7a42630bddabd19e0ae8404
/// Input contains `while (true) { }` - infinite loop causing fuzz timeout.
/// Fixed by checking execution time inside while-loop body.
const TIMEOUT_RUNTIME_INFINITE_LOOP: &str = "while (true) { }";

/// Runtime timeout: timeout-f7f5a160435e361f569e05ab54768b9bcfcf0f5d
/// Long chain of && with repetitive patterns caused 886s timeout.
/// Fixed by timeout check in evaluate_expression and 64KB input limit in fuzz harness.
const TIMEOUT_RUNTIME_LONG_AND_CHAIN: &str = "lhh=B=hhfPPPPPPPhhhPPPPPPPPPPhPPPPhhhhhhhD=\"=\"&&!\"\"\"=\"&&\"\"lD=BPPh=hhhPPPPPPPhhhPPPPPPPh=hh=B=hhhhhhjuu=Bi=\"P\"hPPPPPhhhhhhhD=\"=\"&&!\"\"\"=\"&&\"\"lD=BPPPPhPPPPPPPh=hh=B=hhhhhhjuu=Bi=\"P\"hPPPPPhhhhhhhDlhh=B=hhfPPPPPPPhhhPPPPPPPPPPhPPPPhhhhhhhD=\"=\"&&!\"\"\"=\"&&\"\"lD=BPPh=hhhPPPPPPPhhhPPPPPPPh=hh=B=hhhhhhjuu=Bi=\"P\"hPPPPPhhhhhhhD=\"=\"&&!\"\"\"=\"&&\"\"lD=BPPPPhPPPPPPPh=hh=B=hhhhhhjuu=Bi=\"P\"hPPPPPhhhhhhhD=\"=\"&&!\"\"\"=\"&&\"\"";

#[test]
fn test_fuzz_runtime_timeout_long_and_chain_no_panic() {
    // Regression: timeout-f7f5a160435e361f569e05ab54768b9bcfcf0f5d
    // Long && chain caused 886s timeout. Short-circuit and timeout check should prevent hang.
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let lexer = Lexer::new(TIMEOUT_RUNTIME_LONG_AND_CHAIN);
            if let Ok(tokens_with_pos) = lexer.tokenize_with_positions_immutable() {
                let mut parser = Parser::new_with_positions(tokens_with_pos);
                if let Ok(program) = parser.parse() {
                    let mut runtime = Runtime::new();
                    let _ = runtime.execute_program(program, None);
                    // Must not panic or hang - short-circuit + timeout should complete within 10s
                }
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn test_fuzz_runtime_timeout_infinite_loop_returns_error() {
    // Regression: timeout-acecfa5bd1abe812a7a42630bddabd19e0ae8404
    // while(true){} must not hang - runtime should return ExecutionTimeout within ~10s.
    let lexer = Lexer::new(TIMEOUT_RUNTIME_INFINITE_LOOP);
    if let Ok(tokens_with_pos) = lexer.tokenize_with_positions_immutable() {
        let mut parser = Parser::new_with_positions(tokens_with_pos);
        if let Ok(program) = parser.parse() {
            let mut runtime = Runtime::new();
            let result = runtime.execute_program(program, None);
            assert!(
                result.is_err(),
                "Infinite loop should return ExecutionTimeout"
            );
            let err = result.unwrap_err();
            assert!(
                err.to_string().to_lowercase().contains("timeout"),
                "Expected ExecutionTimeout, got: {}",
                err
            );
        }
    }
}

#[test]
fn test_fuzz_runtime_crash_stack_overflow_and_no_panic() {
    // Regression: crash-3a95ae7b15d7cfc2f50254b2448814f6b406ea39
    // Long chain of && caused AddressSanitizer stack-overflow in evaluate_expression.
    // Fixed by adding MAX_EVAL_DEPTH (128). Must not panic.
    // Use 8MB stack - default (2MB) overflows before recursion limit kicks in.
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let lexer = Lexer::new(CRASH_RUNTIME_STACK_OVERFLOW_AND);
            if let Ok(tokens_with_pos) = lexer.tokenize_with_positions_immutable() {
                let mut parser = Parser::new_with_positions(tokens_with_pos);
                if let Ok(program) = parser.parse() {
                    let mut runtime = Runtime::new();
                    let result = runtime.execute_program(program, None);
                    // Must not panic. Result may be Ok or Err (recursion depth exceeded).
                    assert!(
                        result.is_ok()
                            || result
                                .as_ref()
                                .unwrap_err()
                                .to_string()
                                .contains("recursion")
                    );
                }
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

/// Parser crash: crash-161522ed7f1f2ac04d9aa2e317697ce14c287338
/// Fuzzer-generated "try { }" -like input with typos; previously panicked, now must return Err.
const CRASH_PARSER_161522ED: &str = include_str!("fixtures/fuzz/crash_parser_161522ed.txt");

/// Parser crash: crash-4a6efcfc0987cd7ded43c9b78bb300e7e63953c4
/// Same class as 161522ed (deep try { } nesting, unclosed); must return Err, not panic.
const CRASH_PARSER_4A6EFCFC: &str = include_str!("fixtures/fuzz/crash_parser_4a6efcfc.txt");

/// Runtime slow unit: slow-unit-a36a588e7961eb86bef4a0d3f2553cea65b8d045
/// Contains while (true) { ... }; must not hang - runtime should return ExecutionTimeout.
const SLOW_UNIT_RUNTIME_A36A588E: &str = include_str!("fixtures/fuzz/slow_unit_a36a588e.dal");

#[test]
fn test_fuzz_parser_crash_161522ed_no_panic() {
    // Regression: crash-161522ed7f1f2ac04d9aa2e317697ce14c287338
    // Parser must not panic; must return Err (e.g. "Unexpected end of file. Expected: }").
    // Use 8MB stack - deep try { } nesting can overflow default stack before limit.
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let lexer = Lexer::new(CRASH_PARSER_161522ED);
            let tokens_with_pos = match lexer.tokenize_with_positions_immutable() {
                Ok(t) => t,
                Err(_) => return,
            };
            let mut parser = Parser::new_with_positions(tokens_with_pos);
            let result = parser.parse();
            assert!(
                result.is_err(),
                "Parser should return error for this input, not panic"
            );
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn test_fuzz_parser_crash_4a6efcfc_no_panic() {
    // Regression: crash-4a6efcfc0987cd7ded43c9b78bb300e7e63953c4
    // Same class as 161522ed; parser must return Err, not panic. Use 8MB stack.
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let lexer = Lexer::new(CRASH_PARSER_4A6EFCFC);
            let tokens_with_pos = match lexer.tokenize_with_positions_immutable() {
                Ok(t) => t,
                Err(_) => return,
            };
            let mut parser = Parser::new_with_positions(tokens_with_pos);
            let result = parser.parse();
            assert!(
                result.is_err(),
                "Parser should return error for this input, not panic"
            );
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn test_fuzz_runtime_slow_unit_a36a588e_no_panic() {
    // Regression: slow-unit-a36a588e7961eb86bef4a0d3f2553cea65b8d045
    // Input contains while (true) { ... }. Must not hang; runtime should return ExecutionTimeout.
    let lexer = Lexer::new(SLOW_UNIT_RUNTIME_A36A588E);
    let tokens_with_pos = match lexer.tokenize_with_positions_immutable() {
        Ok(t) => t,
        Err(_) => return,
    };
    let mut parser = Parser::new_with_positions(tokens_with_pos);
    let program = match parser.parse() {
        Ok(p) => p,
        Err(_) => return,
    };
    let mut runtime = Runtime::new();
    let result = runtime.execute_program(program, None);
    // Must not panic. Expect Err(ExecutionTimeout) for infinite loop.
    assert!(
        result.is_err(),
        "Infinite loop should return ExecutionTimeout, got: {:?}",
        result
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.to_lowercase().contains("timeout"),
        "Expected timeout error, got: {}",
        err_msg
    );
}

/// Resolve COO server path across old/new repository layouts.
fn resolve_coo_server_path() -> Option<std::path::PathBuf> {
    let candidates = [
        std::path::Path::new("COO/server.dal").to_path_buf(),
        std::path::Path::new("../COO/server.dal").to_path_buf(),
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("COO/server.dal"),
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../COO/server.dal"),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

/// Server registers all 28 @route handlers (runtime workaround for nested functions).
#[test]
fn test_server_registers_all_routes() {
    use dist_agent_lang::execute_dal_and_extract_handlers_with_path;
    let Some(path) = resolve_coo_server_path() else {
        return;
    };
    let source = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return,
    };
    let (user_functions, _, _, _) = match execute_dal_and_extract_handlers_with_path(&source, &path)
    {
        Ok(x) => x,
        Err(e) => panic!("execute failed: {}", e),
    };
    let route_count = user_functions
        .values()
        .filter(|f| f.attributes.iter().any(|a| a.name == "@route"))
        .count();
    assert!(
        route_count >= 28,
        "Expected >=28 route handlers registered, got {:?}",
        route_count
    );
}

/// Parse server.dal and count functions with @route (top-level and nested). Expect 28.
#[test]
fn test_route_attributes_count() {
    use dist_agent_lang::parser::ast::{BlockStatement, Statement};
    let Some(path) = resolve_coo_server_path() else {
        return;
    };
    let source = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return, // skip if COO/ app tree not present
    };
    let program = match dist_agent_lang::parse_source(&source) {
        Ok(p) => p,
        Err(e) => panic!("parse failed: {}", e),
    };
    fn count_route_in_block(block: &BlockStatement) -> usize {
        let mut n = 0;
        for stmt in &block.statements {
            n += count_route_in_stmt(stmt);
        }
        n
    }
    fn count_route_in_stmt(stmt: &Statement) -> usize {
        match stmt {
            Statement::Function(f) => {
                let has = f.attributes.iter().any(|a| a.name == "@route");
                let mut n = if has { 1 } else { 0 };
                n += count_route_in_block(&f.body);
                n
            }
            Statement::Try(t) => {
                let mut n = count_route_in_block(&t.try_block);
                for c in &t.catch_blocks {
                    n += count_route_in_block(&c.body);
                }
                if let Some(ref fin) = t.finally_block {
                    n += count_route_in_block(fin);
                }
                n
            }
            Statement::Block(b) => count_route_in_block(b),
            Statement::If(i) => {
                count_route_in_block(&i.consequence)
                    + i.alternative
                        .as_ref()
                        .map(count_route_in_block)
                        .unwrap_or(0)
            }
            Statement::While(w) => count_route_in_block(&w.body),
            Statement::ForIn(f) => count_route_in_block(&f.body),
            Statement::Match(m) => m
                .cases
                .iter()
                .map(|c| count_route_in_block(&c.body))
                .sum::<usize>(),
            _ => 0,
        }
    }
    let mut with_route = 0;
    for stmt in &program.statements {
        with_route += count_route_in_stmt(stmt);
    }
    assert!(
        with_route >= 28,
        "Expected >=28 functions with @route, got {}. Parser may drop attributes after first 14.",
        with_route
    );
}
