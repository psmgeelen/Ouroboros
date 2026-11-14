locals {
  devops_services = [
    "iam.googleapis.com",
    "iamcredentials.googleapis.com",
    "cloudresourcemanager.googleapis.com",
    "sts.googleapis.com",
    "storage.googleapis.com",
    "artifactregistry.googleapis.com",
    "serviceusage.googleapis.com",
  ]
  runtime_services = [
    "run.googleapis.com",
    "compute.googleapis.com",
    "secretmanager.googleapis.com",
    "cloudtasks.googleapis.com",
    "artifactregistry.googleapis.com",
    "iam.googleapis.com",
    "storage.googleapis.com",
    "certificatemanager.googleapis.com",
    "serviceusage.googleapis.com",
  ]
}

resource "google_project_service" "devops" {
  for_each           = toset(local.devops_services)
  project            = var.devops_project_id
  service            = each.value
  disable_on_destroy = false
}

resource "google_project_service" "dev" {
  for_each           = toset(local.runtime_services)
  project            = var.dev_project_id
  service            = each.value
  disable_on_destroy = false
}

resource "google_project_service" "prod" {
  for_each           = toset(local.runtime_services)
  project            = var.prod_project_id
  service            = each.value
  disable_on_destroy = false
}

resource "google_storage_bucket" "tfstate" {
  project                     = var.devops_project_id
  name                        = "ouroboros-devops-tfstate"
  location                    = var.region
  uniform_bucket_level_access = true
  public_access_prevention    = "enforced"
  force_destroy               = false

  versioning {
    enabled = true
  }

  depends_on = [google_project_service.devops]
}

resource "google_artifact_registry_repository" "relay" {
  project       = var.devops_project_id
  location      = var.region
  repository_id = "ouroboros"
  format        = "DOCKER"
  description   = "Relay images. The registry does not store cycle history."
  depends_on    = [google_project_service.devops]
}

resource "google_iam_workload_identity_pool" "github" {
  project                   = var.devops_project_id
  workload_identity_pool_id = "github"
  display_name              = "GitHub Actions"
  depends_on                = [google_project_service.devops]
}

resource "google_iam_workload_identity_pool_provider" "github" {
  project                            = var.devops_project_id
  workload_identity_pool_id          = google_iam_workload_identity_pool.github.workload_identity_pool_id
  workload_identity_pool_provider_id = "github"
  display_name                       = "GitHub"
  attribute_mapping = {
    "google.subject"       = "assertion.sub"
    "attribute.repository" = "assertion.repository"
    "attribute.ref"        = "assertion.ref"
  }
  attribute_condition = "assertion.repository == \"psmgeelen/Ouroboros\""
  oidc {
    issuer_uri = "https://token.actions.githubusercontent.com"
  }
}

resource "google_service_account" "github_deploy_dev" {
  project      = var.dev_project_id
  account_id   = "github-deploy-dev"
  display_name = "GitHub deploy dev"
}

resource "google_service_account" "github_deploy_prod" {
  project      = var.prod_project_id
  account_id   = "github-deploy-prod"
  display_name = "GitHub deploy prod"
}

resource "google_service_account" "relay_runtime_dev" {
  project      = var.dev_project_id
  account_id   = "relay-runtime-dev"
  display_name = "Relay runtime dev"
}

resource "google_service_account" "relay_runtime_prod" {
  project      = var.prod_project_id
  account_id   = "relay-runtime-prod"
  display_name = "Relay runtime prod"
}

resource "google_service_account_iam_member" "wif_dev" {
  service_account_id = google_service_account.github_deploy_dev.name
  role               = "roles/iam.workloadIdentityUser"
  member             = "principal://iam.googleapis.com/projects/${var.devops_project_number}/locations/global/workloadIdentityPools/github/subject/repo:psmgeelen/Ouroboros:ref:refs/heads/dev"
}

resource "google_service_account_iam_member" "wif_prod" {
  service_account_id = google_service_account.github_deploy_prod.name
  role               = "roles/iam.workloadIdentityUser"
  member             = "principal://iam.googleapis.com/projects/${var.devops_project_number}/locations/global/workloadIdentityPools/github/subject/repo:psmgeelen/Ouroboros:ref:refs/heads/prod"
}

locals {
  deploy_roles = [
    "roles/run.admin",
    "roles/compute.admin",
    "roles/secretmanager.admin",
    "roles/cloudtasks.admin",
    "roles/storage.admin",
    "roles/iam.serviceAccountUser",
    "roles/artifactregistry.reader",
  ]
}

resource "google_project_iam_member" "deploy_dev" {
  for_each = toset(local.deploy_roles)
  project  = var.dev_project_id
  role     = each.value
  member   = "serviceAccount:${google_service_account.github_deploy_dev.email}"
}

resource "google_project_iam_member" "deploy_prod" {
  for_each = toset(local.deploy_roles)
  project  = var.prod_project_id
  role     = each.value
  member   = "serviceAccount:${google_service_account.github_deploy_prod.email}"
}

resource "google_artifact_registry_repository_iam_member" "deploy_dev_writer" {
  project    = var.devops_project_id
  location   = var.region
  repository = google_artifact_registry_repository.relay.repository_id
  role       = "roles/artifactregistry.writer"
  member     = "serviceAccount:${google_service_account.github_deploy_dev.email}"
}

resource "google_artifact_registry_repository_iam_member" "deploy_prod_writer" {
  project    = var.devops_project_id
  location   = var.region
  repository = google_artifact_registry_repository.relay.repository_id
  role       = "roles/artifactregistry.writer"
  member     = "serviceAccount:${google_service_account.github_deploy_prod.email}"
}

resource "google_artifact_registry_repository_iam_member" "runtime_dev_reader" {
  project    = var.devops_project_id
  location   = var.region
  repository = google_artifact_registry_repository.relay.repository_id
  role       = "roles/artifactregistry.reader"
  member     = "serviceAccount:${google_service_account.relay_runtime_dev.email}"
}

resource "google_artifact_registry_repository_iam_member" "runtime_prod_reader" {
  project    = var.devops_project_id
  location   = var.region
  repository = google_artifact_registry_repository.relay.repository_id
  role       = "roles/artifactregistry.reader"
  member     = "serviceAccount:${google_service_account.relay_runtime_prod.email}"
}

resource "google_storage_bucket_iam_member" "state_dev" {
  bucket = google_storage_bucket.tfstate.name
  role   = "roles/storage.objectAdmin"
  member = "serviceAccount:${google_service_account.github_deploy_dev.email}"
}

resource "google_storage_bucket_iam_member" "state_prod" {
  bucket = google_storage_bucket.tfstate.name
  role   = "roles/storage.objectAdmin"
  member = "serviceAccount:${google_service_account.github_deploy_prod.email}"
}

resource "google_service_account_iam_member" "runtime_dev_user" {
  service_account_id = google_service_account.relay_runtime_dev.name
  role               = "roles/iam.serviceAccountUser"
  member             = "serviceAccount:${google_service_account.github_deploy_dev.email}"
}

resource "google_service_account_iam_member" "runtime_prod_user" {
  service_account_id = google_service_account.relay_runtime_prod.name
  role               = "roles/iam.serviceAccountUser"
  member             = "serviceAccount:${google_service_account.github_deploy_prod.email}"
}
