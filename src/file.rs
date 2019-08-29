pub type BudgetFile = std::fs::File;

pub fn create_budget_file(filepath: &str) -> std::io::Result<BudgetFile> {
    std::fs::File::create(&format!("{}.bud", filepath))
}

pub fn delete_budget_file(filepath: &str) -> std::io::Result<()> {
    std::fs::remove_file(&format!("{}.bud", filepath))
}
