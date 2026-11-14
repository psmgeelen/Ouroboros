variable "region" {
  type    = string
  default = "europe-west1"
}

variable "devops_project_id" {
  type    = string
  default = "ouroboros-devops"
}

variable "dev_project_id" {
  type    = string
  default = "ouroboros-dev-514714"
}

variable "prod_project_id" {
  type    = string
  default = "ouroboros-prod-514714"
}

variable "devops_project_number" {
  type        = string
  description = "Numeric project number of ouroboros-devops, used in the Workload Identity principal."
}
