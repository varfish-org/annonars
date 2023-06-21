# Mangement of the GitHub project.

resource "github_repository" "annonars" {
  name        = "annonars"
  description = "Genome annotation based on Rust and RocksDB"

  has_issues = true
  visibility = "public"

  allow_rebase_merge     = false
  allow_merge_commit     = false
  delete_branch_on_merge = true

  squash_merge_commit_message = "BLANK"
  squash_merge_commit_title   = "PR_TITLE"

  template {
    owner                = "bihealth"
    repository           = "tpl-rs"
    include_all_branches = true
  }
}
