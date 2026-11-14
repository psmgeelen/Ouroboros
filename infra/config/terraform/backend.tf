# Backend is filled in after the bootstrap bucket exists.
# The first bootstrap apply uses a local backend, then state moves to
# gs://ouroboros-devops-tfstate. See docs/bootstrap.md.
terraform {
  backend "local" {}
}
