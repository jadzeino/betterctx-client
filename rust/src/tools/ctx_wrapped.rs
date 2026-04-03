use crate::core::wrapped::WrappedReport;

pub fn handle(period: &str) -> String {
    let report = WrappedReport::generate(period);
    report.format_compact()
}
