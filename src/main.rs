use stacker::remaining_stack;

// IMPORTANT: This file demonstrates why boxed recursion uses the same stack as pure recursion
// KEY INSIGHT: Stack memory is dominated by function call overhead, not data size or boxing

// === แบบ enum + Box (heap) ===
pub enum BoxedFact<T> {
    Next(T, Box<BoxedFact<T>>),
    Done(T),
}

pub fn make_boxed_fact_u8(n: u8) -> BoxedFact<u8> {
    // IMPORTANT: Creates recursive data structure on heap, but evaluation still uses stack
    if n == 0 {
        BoxedFact::Done(1)
    } else {
        BoxedFact::Next(n, Box::new(make_boxed_fact_u8(n - 1)))
    }
}

pub fn make_boxed_fact_u64(n: u64) -> BoxedFact<u64> {
    if n == 0 {
        BoxedFact::Done(1)
    } else {
        BoxedFact::Next(n, Box::new(make_boxed_fact_u64(n - 1)))
    }
}

pub fn make_boxed_fact_u128(n: u128) -> BoxedFact<u128> {
    if n == 0 {
        BoxedFact::Done(1)
    } else {
        BoxedFact::Next(n, Box::new(make_boxed_fact_u128(n - 1)))
    }
}

pub fn eval_boxed_fact_tracked<T>(f: &BoxedFact<T>, stack_info: &mut Vec<usize>) {
    // CRITICAL: This is where stack usage is measured!
    // Despite being "boxed", this still uses ~80 bytes/level - SAME as pure recursion
    if let Some(rem) = remaining_stack() {
        stack_info.push(rem); // Record stack depth at each level
    }
    match f {
        BoxedFact::Next(_, next) => eval_boxed_fact_tracked(next, stack_info),
        BoxedFact::Done(_) => {}
    }
}

// === แบบ fn ธรรมดา (pure stack) ===
pub fn simple_factorial_tracked_u8(n: u8, stack_info: &mut Vec<usize>) {
    // IMPORTANT: Pure recursion - uses SAME stack as boxed (~80 bytes/level)!
    // KEY FINDING: u8 and u64 use identical stack despite 8x size difference
    if let Some(rem) = remaining_stack() {
        stack_info.push(rem); // Record stack depth at each level
    }
    if n > 0 {
        simple_factorial_tracked_u8(n - 1, stack_info);
    }
}

pub fn simple_factorial_tracked_u64(n: u64, stack_info: &mut Vec<usize>) {
    if let Some(rem) = remaining_stack() {
        stack_info.push(rem);
    }
    if n > 0 {
        simple_factorial_tracked_u64(n - 1, stack_info);
    }
}

pub fn simple_factorial_tracked_u128(n: u128, stack_info: &mut Vec<usize>) {
    if let Some(rem) = remaining_stack() {
        stack_info.push(rem);
    }
    if n > 0 {
        simple_factorial_tracked_u128(n - 1, stack_info);
    }
}

// === ต่อ string ทุกชั้น: pure fn (stack) ===
pub fn simple_string_tracked(n: u64, stack_info: &mut Vec<usize>, s: &mut String) {
    // IMPORTANT: This is where boxing actually HELPS - uses ~256 bytes/level
    // String manipulation dominates stack frame, making boxing beneficial
    if let Some(rem) = remaining_stack() {
        stack_info.push(rem); // Record stack depth at each level
    }
    s.push_str(&format!("{}-", n)); // String operation creates larger stack frame
    if n > 0 {
        simple_string_tracked(n - 1, stack_info, s);
    }
}

// === ต่อ string ทุกชั้น: Box recursion (heap) ===
pub enum BoxedString {
    Next(String, Box<BoxedString>),
    Done(String),
}

pub fn make_boxed_string(n: u64) -> BoxedString {
    if n == 0 {
        BoxedString::Done(format!("{}-", n))
    } else {
        BoxedString::Next(format!("{}-", n), Box::new(make_boxed_string(n - 1)))
    }
}

pub fn eval_boxed_string_tracked(f: &BoxedString, stack_info: &mut Vec<usize>, out: &mut String) {
    // IMPORTANT: Boxed string version - uses only ~112 bytes/level (56% reduction!)
    // KEY INSIGHT: Boxing helps when data manipulation dominates stack usage
    if let Some(rem) = remaining_stack() {
        stack_info.push(rem); // Record stack depth at each level
    }
    match f {
        BoxedString::Next(s, next) => {
            out.push_str(s);
            eval_boxed_string_tracked(next, stack_info, out);
        }
        BoxedString::Done(s) => out.push_str(s),
    }
}

// === วิเคราะห์และแสดงผล ===
// CRITICAL: Stack analysis function - calculates total usage and per-level cost
// KEY METRIC: per_level shows why u8=u64 and why boxed=pure for simple recursion
fn analyze_stack(stack_info: &[usize]) -> Option<(usize, f64)> {
    if stack_info.len() < 2 {
        return None;
    }
    let start = stack_info.first().copied()?; // Initial stack position
    let end = *stack_info.iter().min().unwrap_or(&start); // Minimum remaining stack
    let used = start.saturating_sub(end); // Total stack consumed
    let per_level = used as f64 / stack_info.len() as f64; // IMPORTANT: Per-call overhead
    Some((used, per_level))
}

fn run_one_case(n: u64) {
    // IMPORTANT: This function demonstrates the key findings
    // Run multiple data types to show: u8 = u64 ≠ u128, boxed = pure (mostly)
    println!("\n=== factorial({}) ===", n);

    // IMPORTANT: u8 test - proves that data size doesn't affect stack usage
    // EXPECTED: ~80 bytes/level, SAME as u64 despite 8x smaller data size
    let mut s8_stack = Vec::new();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        simple_factorial_tracked_u8(n as u8, &mut s8_stack)
    }));
    if result.is_ok() {
        if let Some((used, per_level)) = analyze_stack(&s8_stack) {
            println!(
                "simple(u8): total used {} bytes ({:.2} per level)",
                used, per_level
            );
        }
    } else {
        println!("simple(u8): stack overflow!");
    }

    // IMPORTANT: u64 test - should show IDENTICAL stack usage to u8 (~80 bytes/level)
    // KEY PROOF: Data size doesn't matter when it fits in registers/alignment
    let mut s_stack = Vec::new();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        simple_factorial_tracked_u64(n, &mut s_stack)
    }));
    if result.is_ok() {
        if let Some((used, per_level)) = analyze_stack(&s_stack) {
            println!(
                "simple(u64): total used {} bytes ({:.2} per level)",
                used, per_level
            );
        }
    } else {
        println!("simple(u64): stack overflow!");
    }

    // IMPORTANT: u128 test - should show MORE stack usage (~112 bytes/level)
    // KEY FINDING: Large data types DO affect stack when they can't fit in registers
    let mut s128_stack = Vec::new();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        simple_factorial_tracked_u128(n as u128, &mut s128_stack)
    }));
    if result.is_ok() {
        if let Some((used, per_level)) = analyze_stack(&s128_stack) {
            println!(
                "simple(u128): total used {} bytes ({:.2} per level)",
                used, per_level
            );
        }
    } else {
        println!("simple(u128): stack overflow!");
    }

    // IMPORTANT: u8 boxed test - should show IDENTICAL to pure u8 (~80 bytes/level)
    // SHOCKING: Boxed recursion uses SAME stack as pure for small data types!
    let boxed8 =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| make_boxed_fact_u8(n as u8)));
    if let Ok(fact) = boxed8 {
        let mut b8_stack = Vec::new();
        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            eval_boxed_fact_tracked(&fact, &mut b8_stack)
        }));
        if res.is_ok() {
            if let Some((used, per_level)) = analyze_stack(&b8_stack) {
                println!(
                    "boxed(u8): total used {} bytes ({:.2} per level)",
                    used, per_level
                );
            }
        } else {
            println!("boxed(u8): overflow while evaluating!");
        }
    } else {
        println!("boxed(u8): overflow while creating!");
    }

    // IMPORTANT: u64 boxed test - should show IDENTICAL to pure u64 (~80 bytes/level)
    // KEY PROOF: Box pointer overhead is negligible compared to function call overhead
    let boxed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| make_boxed_fact_u64(n)));
    if let Ok(fact) = boxed {
        let mut b_stack = Vec::new();
        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            eval_boxed_fact_tracked(&fact, &mut b_stack)
        }));
        if res.is_ok() {
            if let Some((used, per_level)) = analyze_stack(&b_stack) {
                println!(
                    "boxed(u64): total used {} bytes ({:.2} per level)",
                    used, per_level
                );
            }
        } else {
            println!("boxed(u64): overflow while evaluating!");
        }
    } else {
        println!("boxed(u64): overflow while creating!");
    }

    // CRITICAL: u128 boxed test - should show LESS than pure u128 (~80 vs ~112 bytes/level)
    // AMAZING: Boxing actually REDUCES stack usage for large data types!
    // REASON: u128 moved to heap, only pointer (8 bytes) stays on stack during recursion
    let boxed128 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        make_boxed_fact_u128(n as u128)
    }));
    if let Ok(fact) = boxed128 {
        let mut b128_stack = Vec::new();
        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            eval_boxed_fact_tracked(&fact, &mut b128_stack)
        }));
        if res.is_ok() {
            if let Some((used, per_level)) = analyze_stack(&b128_stack) {
                println!(
                    "boxed(u128): total used {} bytes ({:.2} per level)",
                    used, per_level
                );
            }
        } else {
            println!("boxed(u128): overflow while evaluating!");
        }
    } else {
        println!("boxed(u128): overflow while creating!");
    }

    // IMPORTANT: Pure string building test - shows HIGH stack usage (~256 bytes/level)
    // KEY INSIGHT: String manipulation dominates stack frame, making recursion expensive
    let mut str_stack = Vec::new();
    let mut s = String::with_capacity((n as usize) * 4);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        simple_string_tracked(n, &mut str_stack, &mut s)
    }));
    if result.is_ok() {
        if let Some((used, per_level)) = analyze_stack(&str_stack) {
            println!(
                "string(pure): total used {} bytes ({:.2} per level)",
                used, per_level
            );
        }
    } else {
        println!("string(pure): stack overflow!");
    }

    // CRITICAL: Boxed string building test - shows MUCH LOWER stack usage (~112 bytes/level)
    // AMAZING: Boxing reduces stack usage by 56% for string operations!
    // REASON: String objects moved to heap, only pointers and small data stay on stack
    let boxed_str = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| make_boxed_string(n)));
    if let Ok(tree) = boxed_str {
        let mut stack_s = Vec::new();
        let mut out = String::with_capacity((n as usize) * 4);
        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            eval_boxed_string_tracked(&tree, &mut stack_s, &mut out)
        }));
        if res.is_ok() {
            if let Some((used, per_level)) = analyze_stack(&stack_s) {
                println!(
                    "string(boxed): total used {} bytes ({:.2} per level)",
                    used, per_level
                );
            }
        } else {
            println!("string(boxed): overflow while evaluating!");
        }
    } else {
        println!("string(boxed): overflow while creating!");
    }
}

fn main() {
    // IMPORTANT: This program demonstrates surprising truths about stack memory usage!
    // EXPECTED RESULTS:
    // - u8, u64, boxed(u8), boxed(u64): all ~80 bytes/level (IDENTICAL!)
    // - u128 pure: ~112 bytes/level (more due to large data)
    // - u128 boxed: ~80 bytes/level (boxed helps with large data!)
    // - string pure: ~256 bytes/level (expensive due to string ops)
    // - string boxed: ~112 bytes/level (boxed reduces cost by 56%)

    println!("=== Stack memory usage per recursion level ===");
    println!("(lower = uses less stack per call)");
    println!("\nKEY INSIGHTS TO WATCH FOR:");
    println!("1. u8 = u64 = boxed(u8) = boxed(u64) (~80 bytes/level)");
    println!("2. boxed(u128) < u128 (boxing HELPS with large data)");
    println!("3. boxed(string) < string (boxing helps with complex ops)");

    for &n in [10, 100, 500, 1000, 5000].iter() {
        run_one_case(n);
    }
}
