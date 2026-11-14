variable "project_id" {
  type = string
}

variable "region" {
  type    = string
  default = "europe-west1"
}

variable "image" {
  type        = string
  description = "Relay image in the devops Artifact Registry."
}

variable "runtime_service_account" {
  type        = string
  description = "Email of the runtime service account created by bootstrap."
}

variable "pepper" {
  type        = string
  sensitive   = true
  description = "Account-code HMAC pepper. Not the cycle data."
}
