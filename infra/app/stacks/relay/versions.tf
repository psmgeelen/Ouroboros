terraform {
  required_version = ">= 1.6.0"
  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 6.0"
    }
  }
  backend "gcs" {
    bucket = "ouroboros-devops-tfstate"
  }
}

provider "google" {
  project = var.project_id
  region  = var.region
}
