output "wif_provider" {
  description = "Workload Identity provider resource name for GitHub Actions."
  value       = google_iam_workload_identity_pool_provider.github.name
}

output "github_build" {
  description = "Registry writer for the main build. It cannot deploy Cloud Run."
  value       = google_service_account.github_build.email
}

output "github_deploy_dev" {
  value = google_service_account.github_deploy_dev.email
}

output "github_deploy_prod" {
  value = google_service_account.github_deploy_prod.email
}

output "relay_runtime_dev" {
  value = google_service_account.relay_runtime_dev.email
}

output "relay_runtime_prod" {
  value = google_service_account.relay_runtime_prod.email
}
