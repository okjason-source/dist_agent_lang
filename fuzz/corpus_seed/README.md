# Fuzzing Seed Corpus

This directory contains seed files for fuzzing targets. Seed files are manually curated examples that provide good starting points for fuzzing.

## Directory Structure

- `fuzz_lexer/` - Seed files for the lexer fuzzer (manual seeds only)
- `fuzz_parser/` - Seed files for the parser fuzzer
- `fuzz_runtime/` - Seed files for the runtime fuzzer
- `fuzz_lexer_generated/` - Auto-generated corpus files from previous fuzzing runs (not seeds)

## Seed File Guidelines

### What Makes a Good Seed?

1. **Valid programs** - Should parse/lex correctly
2. **Diverse syntax** - Cover different language features
3. **Edge cases** - Minimal examples, empty files, deeply nested structures
4. **Real-world examples** - Actual code from your codebase
5. **Previously found bugs** - Regression tests for fixed bugs

### Seed Count Recommendations

- **Minimum**: 10-50 diverse seeds per target
- **Optimal**: 50-200 seeds per target
- **Maximum**: Beyond 500-1000, diminishing returns

### Current Status

- `fuzz_lexer`: 3 manual seed files (`.dal` files)
- `fuzz_parser`: 15 seed files
- `fuzz_runtime`: 15 seed files

## Generated Corpus Files

The `fuzz_lexer_generated/` directory contains auto-generated corpus files from libFuzzer runs. These are:
- Hash-named files (SHA-1 hashes as filenames)
- Mutated/corrupted inputs that found new coverage
- Useful for corpus analysis, but not human-readable seeds

These files are kept separate from seeds for clarity.

## Adding New Seeds

1. Copy valid `.dal` files to the appropriate target directory
2. Use descriptive filenames (e.g., `minimal_service.dal`, `edge_case_nested.dal`)
3. Ensure files are valid syntax (they should parse/lex correctly)
4. Document what each seed tests in comments

## Usage

When running fuzzers, libFuzzer will:
1. Start with seed files from `corpus_seed/<target>/`
2. Generate new test cases based on seeds
3. Save interesting inputs to `corpus/<target>/` (not in this directory)
