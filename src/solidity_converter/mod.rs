// Solidity to DAL Converter Module
// Converts Solidity source code to dist_agent_lang format

pub mod parser;
pub mod converter;
pub mod types;
pub mod security;
pub mod generator;
pub mod analyzer;

pub use parser::SolidityParser;
pub use converter::SolidityConverter;
pub use analyzer::ConversionAnalyzer;

use std::path::Path;

/// Convert Solidity file to DAL
pub fn convert_file(input_path: &Path, output_path: &Path) -> Result<String, String> {
    // Read Solidity file
    let solidity_code = std::fs::read_to_string(input_path)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    // Parse Solidity
    let parser = SolidityParser::new();
    let solidity_ast = parser.parse(&solidity_code)?;
    
    // Convert to DAL
    let converter = SolidityConverter::new();
    let dal_ast = converter.convert(solidity_ast)?;
    
    // Generate DAL code
    let generator = generator::DALGenerator::new();
    let dal_code = generator.generate(dal_ast)?;
    
    // Write output
    std::fs::write(output_path, &dal_code)
        .map_err(|e| format!("Failed to write output file: {}", e))?;
    
    Ok(dal_code)
}

/// Analyze Solidity file for conversion compatibility
pub fn analyze_file(input_path: &Path) -> Result<analyzer::AnalysisReport, String> {
    let solidity_code = std::fs::read_to_string(input_path)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    let parser = SolidityParser::new();
    let solidity_ast = parser.parse(&solidity_code)?;
    
    let analyzer = ConversionAnalyzer::new();
    analyzer.analyze(solidity_ast)
}

