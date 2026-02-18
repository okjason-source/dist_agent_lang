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
            let _ = runtime.execute_program(program);
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
            let _ = runtime.execute_program(program);
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
            let _ = runtime.execute_program(program);
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
                    let _ = runtime.execute_program(program);
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
                    let _ = runtime.execute_program(program);
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
            let result = runtime.execute_program(program);
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
                    let result = runtime.execute_program(program);
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
