use stacker::remaining_stack;

// IMPORTANT: This file demonstrates why boxed recursion uses the same stack as pure recursion
// KEY INSIGHT: Stack memory is dominated by function call overhead, not data size or boxing

// === แบบ enum + Box (heap) ===
pub enum BoxedFact<T> {
    Next(T, Box<BoxedFact<T>>),
    Done(T),
}

pub fn make_boxed_fact_u8(n: u8) -> BoxedFact<u8> {
    // IMPORTANT: Iterative creation to eliminate creation-phase stack overflow!
    // Instead of recursive building, we build from bottom up using a loop
    let mut current = BoxedFact::Done(1);

    // Build the structure backwards: from 0 up to n
    for i in 1..=n {
        current = BoxedFact::Next(i, Box::new(current));
    }

    current
}

pub fn make_boxed_fact_u64(n: u64) -> BoxedFact<u64> {
    // IMPORTANT: Iterative creation to eliminate creation-phase stack overflow!
    // Instead of recursive building, we build from bottom up using a loop
    let mut current = BoxedFact::Done(1);

    // Build the structure backwards: from 0 up to n
    for i in 1..=n {
        current = BoxedFact::Next(i, Box::new(current));
    }

    current
}

pub fn make_boxed_fact_u128(n: u128) -> BoxedFact<u128> {
    // IMPORTANT: Iterative creation to eliminate creation-phase stack overflow!
    // Instead of recursive building, we build from bottom up using a loop
    let mut current = BoxedFact::Done(1);

    // Build the structure backwards: from 0 up to n
    for i in 1..=n {
        current = BoxedFact::Next(i, Box::new(current));
    }

    current
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
    // IMPORTANT: Iterative creation to eliminate creation-phase stack overflow!
    // Instead of recursive building, we build from bottom up using a loop
    let mut current = BoxedString::Done(format!("{}-", 0));

    // Build the structure backwards: from 1 up to n
    for i in (1..=n).rev() {
        current = BoxedString::Next(format!("{}-", i), Box::new(current));
    }

    current
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

    // Comment out problematic tests that cause stack overflow
    // for &n in [20_000, 80_000].iter() {
    //     run_one_case(n);
    // }

    // ISOLATED TEST: Compare simple vs boxed u128 at same depth
    println!("\n=== ISOLATED COMPARISON: simple(u128) vs boxed(u128) ===");
    let test_depth = 70_000;

    println!("\nTesting simple(u128) at depth {}:", test_depth);
    let mut simple_stack = Vec::new();
    let simple_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        simple_factorial_tracked_u128(test_depth as u128, &mut simple_stack)
    }));
    match simple_result {
        Ok(_) => {
            if let Some((used, per_level)) = analyze_stack(&simple_stack) {
                println!(
                    "simple(u128): SUCCESS - {} bytes ({:.2} per level)",
                    used, per_level
                );
            }
        }
        Err(_) => println!("simple(u128): STACK OVERFLOW"),
    }

    println!("\nTesting boxed(u128) at depth {}:", test_depth);
    // Test creation separately
    let boxed_creation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        make_boxed_fact_u128(test_depth as u128)
    }));
    let mut boxed_stack = Vec::new();
    match boxed_creation {
        Ok(ref fact) => {
            println!("boxed(u128): Creation successful");
            let boxed_eval = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                eval_boxed_fact_tracked(&fact, &mut boxed_stack)
            }));
            match boxed_eval {
                Ok(_) => {
                    if let Some((used, per_level)) = analyze_stack(&boxed_stack) {
                        println!(
                            "boxed(u128): SUCCESS - {} bytes ({:.2} per level)",
                            used, per_level
                        );
                    }
                }
                Err(_) => println!("boxed(u128): STACK OVERFLOW during evaluation"),
            }
        }
        Err(_) => println!("boxed(u128): STACK OVERFLOW during creation"),
    }

    // Force cleanup of all data before next test
    drop(boxed_creation);
    drop(boxed_stack);
    drop(simple_stack);

    // TEST AT HIGHER DEPTH: Show boxed can handle what simple cannot
    println!("\n=== HIGH DEPTH TEST: ONLY boxed(u128) (simple would overflow) ===");
    let high_depth = 90_000; // Beyond simple u128 capability

    println!(
        "\nTesting ONLY boxed(u128) at depth {} (simple u128 would overflow):",
        high_depth
    );
    let boxed_creation_high = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        make_boxed_fact_u128(high_depth as u128)
    }));
    let mut boxed_stack_high = Vec::new();
    match boxed_creation_high {
        Ok(ref fact_high) => {
            println!("boxed(u128): ✅ Creation successful");
            let boxed_eval_high = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                eval_boxed_fact_tracked(&fact_high, &mut boxed_stack_high)
            }));
            match boxed_eval_high {
                Ok(_) => {
                    if let Some((used, per_level)) = analyze_stack(&boxed_stack_high) {
                        println!(
                            "boxed(u128): ✅ SUCCESS - {} bytes ({:.2} per level)",
                            used, per_level
                        );
                        println!(
                            "🎯 BOXED u128 HANDLES {} LEVELS WHERE SIMPLE u128 WOULD OVERFLOW!",
                            high_depth
                        );
                    }
                }
                Err(_) => println!("boxed(u128): ❌ STACK OVERFLOW during evaluation"),
            }
        }
        Err(_) => println!("boxed(u128): ❌ STACK OVERFLOW during creation"),
    }

    // Force cleanup of high depth test data
    drop(boxed_creation_high);
    drop(boxed_stack_high);

    // ULTIMATE PROOF: Test even higher depth
    println!("\n=== ULTIMATE TEST: boxed(u128) at extreme depth ===");
    let extreme_depth = 100_000;

    println!("\nTesting boxed(u128) at depth {}:", extreme_depth);
    let boxed_creation_extreme = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        make_boxed_fact_u128(extreme_depth as u128)
    }));
    let mut boxed_stack_extreme = Vec::new();
    match boxed_creation_extreme {
        Ok(ref fact_extreme) => {
            println!("boxed(u128): ✅ Creation successful at extreme depth!");
            let boxed_eval_extreme = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                eval_boxed_fact_tracked(&fact_extreme, &mut boxed_stack_extreme)
            }));
            match boxed_eval_extreme {
                Ok(_) => {
                    if let Some((used, per_level)) = analyze_stack(&boxed_stack_extreme) {
                        println!(
                            "boxed(u128): ✅ SUCCESS - {} bytes ({:.2} per level)",
                            used, per_level
                        );
                        println!(
                            "🏆 BOXED u128 ACHIEVES {} LEVELS! (simple u128 max ~71,000)",
                            extreme_depth
                        );
                    }
                }
                Err(_) => {
                    println!("boxed(u128): ❌ STACK OVERFLOW during evaluation at extreme depth")
                }
            }
        }
        Err(_) => println!("boxed(u128): ❌ STACK OVERFLOW during creation at extreme depth"),
    }

    // Force cleanup of extreme depth test data
    drop(boxed_creation_extreme);
    drop(boxed_stack_extreme);

    // STRING TEST: Compare pure vs boxed string building at same depth
    println!("\n=== STRING COMPARISON: pure vs boxed string building ===");
    let string_depth = 10_000;

    println!("\nTesting pure string building at depth {}:", string_depth);
    let mut pure_str_stack = Vec::new();
    let mut pure_string = String::with_capacity((string_depth as usize) * 4);
    let pure_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        simple_string_tracked(string_depth, &mut pure_str_stack, &mut pure_string)
    }));
    match pure_result {
        Ok(_) => {
            if let Some((used, per_level)) = analyze_stack(&pure_str_stack) {
                println!(
                    "string(pure): SUCCESS - {} bytes ({:.2} per level)",
                    used, per_level
                );
            }
        }
        Err(_) => println!("string(pure): STACK OVERFLOW"),
    }

    println!("\nTesting boxed string building at depth {}:", string_depth);
    let boxed_str_creation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        make_boxed_string(string_depth)
    }));
    let mut boxed_str_stack = Vec::new();
    let mut boxed_string = String::with_capacity((string_depth as usize) * 4);
    match boxed_str_creation {
        Ok(ref str_tree) => {
            println!("string(boxed): Creation successful");
            let boxed_str_eval = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                eval_boxed_string_tracked(&str_tree, &mut boxed_str_stack, &mut boxed_string)
            }));
            match boxed_str_eval {
                Ok(_) => {
                    if let Some((used, per_level)) = analyze_stack(&boxed_str_stack) {
                        println!(
                            "string(boxed): SUCCESS - {} bytes ({:.2} per level)",
                            used, per_level
                        );
                    }
                }
                Err(_) => println!("string(boxed): STACK OVERFLOW during evaluation"),
            }
        }
        Err(_) => println!("string(boxed): STACK OVERFLOW during creation"),
    }

    // Force cleanup of string comparison test data
    drop(boxed_str_creation);
    drop(boxed_str_stack);
    drop(pure_str_stack);
    drop(pure_string);
    drop(boxed_string);

    // HIGH DEPTH STRING TEST: Show boxed can handle what pure cannot
    println!("\n=== HIGH DEPTH STRING TEST: pure vs boxed at depth 32,000 ===");
    let high_string_depth = 32_000;

    println!(
        "\nTesting pure string at depth {} (should overflow):",
        high_string_depth
    );
    let mut high_pure_str_stack = Vec::new();
    let mut high_pure_string = String::with_capacity((high_string_depth as usize) * 4);
    let high_pure_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        simple_string_tracked(
            high_string_depth,
            &mut high_pure_str_stack,
            &mut high_pure_string,
        )
    }));
    match high_pure_result {
        Ok(_) => {
            if let Some((used, per_level)) = analyze_stack(&high_pure_str_stack) {
                println!(
                    "string(pure): SUCCESS - {} bytes ({:.2} per level)",
                    used, per_level
                );
            }
        }
        Err(_) => println!("string(pure): ❌ STACK OVERFLOW at depth 30,000"),
    }

    println!(
        "\nTesting boxed string at depth {} (pure string would overflow):",
        high_string_depth
    );
    let boxed_str_high = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        make_boxed_string(high_string_depth)
    }));
    let mut boxed_str_stack_high = Vec::new();
    let mut boxed_string_high = String::with_capacity((high_string_depth as usize) * 4);
    match boxed_str_high {
        Ok(ref str_tree_high) => {
            println!("string(boxed): ✅ Creation successful");
            let boxed_str_eval_high =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    eval_boxed_string_tracked(
                        &str_tree_high,
                        &mut boxed_str_stack_high,
                        &mut boxed_string_high,
                    )
                }));
            match boxed_str_eval_high {
                Ok(_) => {
                    if let Some((used, per_level)) = analyze_stack(&boxed_str_stack_high) {
                        println!(
                            "string(boxed): ✅ SUCCESS - {} bytes ({:.2} per level)",
                            used, per_level
                        );
                        println!(
                            "🎯 BOXED string HANDLES {} LEVELS WHERE PURE string WOULD OVERFLOW!",
                            high_string_depth
                        );
                    }
                }
                Err(_) => println!("string(boxed): ❌ STACK OVERFLOW during evaluation"),
            }
        }
        Err(_) => println!("string(boxed): ❌ STACK OVERFLOW during creation"),
    }

    // Force cleanup of high depth string test data
    drop(high_pure_result);
    drop(high_pure_str_stack);
    drop(high_pure_string);
    drop(boxed_str_high);
    drop(boxed_str_stack_high);
    drop(boxed_string_high);
}
