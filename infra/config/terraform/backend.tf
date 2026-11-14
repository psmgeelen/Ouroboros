# Terraspace compiles this into each stack.
# Prefixes match task infra:bootstrap (bootstrap) and task infra:up (dev or prod).
# The bucket is gs://tf-state-ouroboros in ouroboros-devops.
terraform {
  backend "gcs" {
    bucket = "tf-state-ouroboros"
    prefix = "<%= expansion(':MOD_NAME') == 'bootstrap' ? 'bootstrap' : expansion(':ENV') %>"
  }
}
