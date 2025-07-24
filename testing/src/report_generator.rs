// =====================================================================================
// File: testing/src/report_generator.rs
// Description: Test report generation utilities
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use uuid::Uuid;

/// Test execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub id: Uuid,
    pub name: String,
    pub module: String,
    pub test_type: TestType,
    pub status: TestStatus,
    pub duration_ms: u64,
    pub error_message: Option<String>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
}

/// Test type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TestType {
    Unit,
    Integration,
    Benchmark,
    Load,
    Stress,
    EndToEnd,
}

/// Test status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Timeout,
    Error,
}

/// Coverage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageInfo {
    pub module: String,
    pub line_coverage: f64,
    pub branch_coverage: f64,
    pub function_coverage: f64,
    pub total_lines: u32,
    pub covered_lines: u32,
    pub total_branches: u32,
    pub covered_branches: u32,
    pub total_functions: u32,
    pub covered_functions: u32,
}

/// Benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub name: String,
    pub module: String,
    pub mean_time_ns: u64,
    pub std_dev_ns: u64,
    pub min_time_ns: u64,
    pub max_time_ns: u64,
    pub throughput: Option<f64>,
    pub samples: u32,
}

/// Comprehensive test report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReport {
    pub id: Uuid,
    pub generated_at: DateTime<Utc>,
    pub summary: TestSummary,
    pub test_results: Vec<TestResult>,
    pub coverage_info: Vec<CoverageInfo>,
    pub benchmark_results: Vec<BenchmarkResult>,
    pub environment_info: EnvironmentInfo,
    pub configuration: TestConfiguration,
}

/// Test summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSummary {
    pub total_tests: u32,
    pub passed_tests: u32,
    pub failed_tests: u32,
    pub skipped_tests: u32,
    pub timeout_tests: u32,
    pub error_tests: u32,
    pub total_duration_ms: u64,
    pub success_rate: f64,
    pub overall_coverage: f64,
    pub modules_tested: u32,
    pub benchmarks_run: u32,
}

/// Environment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    pub rust_version: String,
    pub cargo_version: String,
    pub os: String,
    pub arch: String,
    pub cpu_cores: u32,
    pub memory_gb: f64,
    pub git_commit: Option<String>,
    pub git_branch: Option<String>,
    pub build_timestamp: DateTime<Utc>,
}

/// Test configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfiguration {
    pub test_timeout: u64,
    pub benchmark_timeout: u64,
    pub coverage_threshold: f64,
    pub parallel_jobs: u32,
    pub modules_tested: Vec<String>,
    pub features_enabled: Vec<String>,
}

/// Test report generator
pub struct TestReportGenerator {
    output_dir: String,
    template_dir: String,
}

impl TestReportGenerator {
    pub fn new(output_dir: String, template_dir: String) -> Self {
        Self {
            output_dir,
            template_dir,
        }
    }

    /// Generate comprehensive test report
    pub fn generate_report(&self, report: &TestReport) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure output directory exists
        fs::create_dir_all(&self.output_dir)?;

        // Generate HTML report
        self.generate_html_report(report)?;

        // Generate JSON report
        self.generate_json_report(report)?;

        // Generate JUnit XML report
        self.generate_junit_xml_report(report)?;

        // Generate coverage report
        self.generate_coverage_report(report)?;

        // Generate benchmark report
        if !report.benchmark_results.is_empty() {
            self.generate_benchmark_report(report)?;
        }

        Ok(())
    }

    /// Generate HTML report
    fn generate_html_report(&self, report: &TestReport) -> Result<(), Box<dyn std::error::Error>> {
        let html_content = self.render_html_template(report)?;
        let output_path = Path::new(&self.output_dir).join("test-report.html");
        fs::write(output_path, html_content)?;
        Ok(())
    }

    /// Generate JSON report
    fn generate_json_report(&self, report: &TestReport) -> Result<(), Box<dyn std::error::Error>> {
        let json_content = serde_json::to_string_pretty(report)?;
        let output_path = Path::new(&self.output_dir).join("test-report.json");
        fs::write(output_path, json_content)?;
        Ok(())
    }

    /// Generate JUnit XML report
    fn generate_junit_xml_report(&self, report: &TestReport) -> Result<(), Box<dyn std::error::Error>> {
        let xml_content = self.render_junit_xml(report)?;
        let output_path = Path::new(&self.output_dir).join("junit-report.xml");
        fs::write(output_path, xml_content)?;
        Ok(())
    }

    /// Generate coverage report
    fn generate_coverage_report(&self, report: &TestReport) -> Result<(), Box<dyn std::error::Error>> {
        let coverage_html = self.render_coverage_html(report)?;
        let output_path = Path::new(&self.output_dir).join("coverage-report.html");
        fs::write(output_path, coverage_html)?;
        Ok(())
    }

    /// Generate benchmark report
    fn generate_benchmark_report(&self, report: &TestReport) -> Result<(), Box<dyn std::error::Error>> {
        let benchmark_html = self.render_benchmark_html(report)?;
        let output_path = Path::new(&self.output_dir).join("benchmark-report.html");
        fs::write(output_path, benchmark_html)?;
        Ok(())
    }

    /// Render HTML template
    fn render_html_template(&self, report: &TestReport) -> Result<String, Box<dyn std::error::Error>> {
        let template = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>RWA Platform Test Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background: #f4f4f4; padding: 20px; border-radius: 5px; }
        .summary { display: flex; gap: 20px; margin: 20px 0; }
        .metric { background: #e9f5ff; padding: 15px; border-radius: 5px; text-align: center; }
        .metric.success { background: #d4edda; }
        .metric.warning { background: #fff3cd; }
        .metric.error { background: #f8d7da; }
        .test-results { margin: 20px 0; }
        .test-item { padding: 10px; border-left: 4px solid #ccc; margin: 5px 0; }
        .test-item.passed { border-left-color: #28a745; }
        .test-item.failed { border-left-color: #dc3545; }
        .test-item.skipped { border-left-color: #ffc107; }
        table { width: 100%; border-collapse: collapse; margin: 20px 0; }
        th, td { padding: 10px; text-align: left; border-bottom: 1px solid #ddd; }
        th { background-color: #f2f2f2; }
    </style>
</head>
<body>
    <div class="header">
        <h1>RWA Platform Test Report</h1>
        <p>Generated: {generated_at}</p>
        <p>Report ID: {report_id}</p>
    </div>

    <div class="summary">
        <div class="metric {success_class}">
            <h3>Success Rate</h3>
            <p>{success_rate:.1}%</p>
        </div>
        <div class="metric">
            <h3>Total Tests</h3>
            <p>{total_tests}</p>
        </div>
        <div class="metric">
            <h3>Coverage</h3>
            <p>{overall_coverage:.1}%</p>
        </div>
        <div class="metric">
            <h3>Duration</h3>
            <p>{total_duration_ms}ms</p>
        </div>
    </div>

    <h2>Test Results by Module</h2>
    <table>
        <thead>
            <tr>
                <th>Module</th>
                <th>Tests</th>
                <th>Passed</th>
                <th>Failed</th>
                <th>Coverage</th>
                <th>Duration</th>
            </tr>
        </thead>
        <tbody>
            {module_rows}
        </tbody>
    </table>

    <h2>Failed Tests</h2>
    <div class="test-results">
        {failed_tests}
    </div>

    <h2>Environment Information</h2>
    <table>
        <tr><td>Rust Version</td><td>{rust_version}</td></tr>
        <tr><td>OS</td><td>{os}</td></tr>
        <tr><td>Architecture</td><td>{arch}</td></tr>
        <tr><td>CPU Cores</td><td>{cpu_cores}</td></tr>
        <tr><td>Memory</td><td>{memory_gb:.1} GB</td></tr>
        <tr><td>Git Commit</td><td>{git_commit}</td></tr>
        <tr><td>Git Branch</td><td>{git_branch}</td></tr>
    </table>
</body>
</html>
        "#;

        // Group results by module
        let mut module_stats: HashMap<String, (u32, u32, u32, f64, u64)> = HashMap::new();
        for result in &report.test_results {
            let entry = module_stats.entry(result.module.clone()).or_insert((0, 0, 0, 0.0, 0));
            entry.0 += 1; // total
            if result.status == TestStatus::Passed {
                entry.1 += 1; // passed
            } else if result.status == TestStatus::Failed {
                entry.2 += 1; // failed
            }
            entry.4 += result.duration_ms; // duration
        }

        // Add coverage info
        for coverage in &report.coverage_info {
            if let Some(entry) = module_stats.get_mut(&coverage.module) {
                entry.3 = coverage.line_coverage;
            }
        }

        let module_rows = module_stats
            .iter()
            .map(|(module, (total, passed, failed, coverage, duration))| {
                format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{:.1}%</td><td>{}ms</td></tr>",
                    module, total, passed, failed, coverage, duration
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let failed_tests = report
            .test_results
            .iter()
            .filter(|r| r.status == TestStatus::Failed)
            .map(|r| {
                format!(
                    r#"<div class="test-item failed">
                        <h4>{} ({})</h4>
                        <p>Duration: {}ms</p>
                        <p>Error: {}</p>
                    </div>"#,
                    r.name,
                    r.module,
                    r.duration_ms,
                    r.error_message.as_deref().unwrap_or("Unknown error")
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let success_class = if report.summary.success_rate >= 95.0 {
            "success"
        } else if report.summary.success_rate >= 80.0 {
            "warning"
        } else {
            "error"
        };

        let rendered = template
            .replace("{generated_at}", &report.generated_at.format("%Y-%m-%d %H:%M:%S UTC").to_string())
            .replace("{report_id}", &report.id.to_string())
            .replace("{success_class}", success_class)
            .replace("{success_rate}", &report.summary.success_rate.to_string())
            .replace("{total_tests}", &report.summary.total_tests.to_string())
            .replace("{overall_coverage}", &report.summary.overall_coverage.to_string())
            .replace("{total_duration_ms}", &report.summary.total_duration_ms.to_string())
            .replace("{module_rows}", &module_rows)
            .replace("{failed_tests}", &failed_tests)
            .replace("{rust_version}", &report.environment_info.rust_version)
            .replace("{os}", &report.environment_info.os)
            .replace("{arch}", &report.environment_info.arch)
            .replace("{cpu_cores}", &report.environment_info.cpu_cores.to_string())
            .replace("{memory_gb}", &report.environment_info.memory_gb.to_string())
            .replace("{git_commit}", &report.environment_info.git_commit.as_deref().unwrap_or("Unknown"))
            .replace("{git_branch}", &report.environment_info.git_branch.as_deref().unwrap_or("Unknown"));

        Ok(rendered)
    }

    /// Render JUnit XML
    fn render_junit_xml(&self, report: &TestReport) -> Result<String, Box<dyn std::error::Error>> {
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push_str(&format!(
            r#"<testsuites name="RWA Platform Tests" tests="{}" failures="{}" time="{:.3}">"#,
            report.summary.total_tests,
            report.summary.failed_tests,
            report.summary.total_duration_ms as f64 / 1000.0
        ));

        // Group by module
        let mut modules: HashMap<String, Vec<&TestResult>> = HashMap::new();
        for result in &report.test_results {
            modules.entry(result.module.clone()).or_default().push(result);
        }

        for (module, tests) in modules {
            let module_failures = tests.iter().filter(|t| t.status == TestStatus::Failed).count();
            let module_duration: u64 = tests.iter().map(|t| t.duration_ms).sum();

            xml.push_str(&format!(
                r#"<testsuite name="{}" tests="{}" failures="{}" time="{:.3}">"#,
                module,
                tests.len(),
                module_failures,
                module_duration as f64 / 1000.0
            ));

            for test in tests {
                xml.push_str(&format!(
                    r#"<testcase name="{}" classname="{}" time="{:.3}">"#,
                    test.name,
                    test.module,
                    test.duration_ms as f64 / 1000.0
                ));

                if test.status == TestStatus::Failed {
                    xml.push_str(&format!(
                        r#"<failure message="{}">{}</failure>"#,
                        test.error_message.as_deref().unwrap_or("Test failed"),
                        test.stderr.as_deref().unwrap_or("")
                    ));
                }

                xml.push_str("</testcase>");
            }

            xml.push_str("</testsuite>");
        }

        xml.push_str("</testsuites>");
        Ok(xml)
    }

    /// Render coverage HTML
    fn render_coverage_html(&self, report: &TestReport) -> Result<String, Box<dyn std::error::Error>> {
        // Simplified coverage report
        let mut html = String::from(r#"
<!DOCTYPE html>
<html>
<head><title>Coverage Report</title></head>
<body>
<h1>Test Coverage Report</h1>
<table border="1">
<tr><th>Module</th><th>Line Coverage</th><th>Branch Coverage</th><th>Function Coverage</th></tr>
        "#);

        for coverage in &report.coverage_info {
            html.push_str(&format!(
                "<tr><td>{}</td><td>{:.1}%</td><td>{:.1}%</td><td>{:.1}%</td></tr>",
                coverage.module,
                coverage.line_coverage,
                coverage.branch_coverage,
                coverage.function_coverage
            ));
        }

        html.push_str("</table></body></html>");
        Ok(html)
    }

    /// Render benchmark HTML
    fn render_benchmark_html(&self, report: &TestReport) -> Result<String, Box<dyn std::error::Error>> {
        // Simplified benchmark report
        let mut html = String::from(r#"
<!DOCTYPE html>
<html>
<head><title>Benchmark Report</title></head>
<body>
<h1>Benchmark Results</h1>
<table border="1">
<tr><th>Benchmark</th><th>Module</th><th>Mean Time (ns)</th><th>Std Dev (ns)</th><th>Throughput</th></tr>
        "#);

        for benchmark in &report.benchmark_results {
            html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                benchmark.name,
                benchmark.module,
                benchmark.mean_time_ns,
                benchmark.std_dev_ns,
                benchmark.throughput.map_or("N/A".to_string(), |t| format!("{:.2}", t))
            ));
        }

        html.push_str("</table></body></html>");
        Ok(html)
    }
}

impl Default for TestReportGenerator {
    fn default() -> Self {
        Self::new(
            "target/test-reports".to_string(),
            "testing/templates".to_string(),
        )
    }
}
