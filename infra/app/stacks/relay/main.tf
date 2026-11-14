resource "google_storage_bucket" "snapshots" {
  name                        = "${var.project_id}-relay-snapshots"
  location                    = var.region
  uniform_bucket_level_access = true
  public_access_prevention    = "enforced"
  force_destroy               = false

  versioning {
    enabled = true
  }

  lifecycle_rule {
    condition {
      num_newer_versions = 3
    }
    action {
      type = "Delete"
    }
  }
}

resource "google_storage_bucket_iam_member" "runtime_snapshots" {
  bucket = google_storage_bucket.snapshots.name
  role   = "roles/storage.objectAdmin"
  member = "serviceAccount:${var.runtime_service_account}"
}

resource "google_secret_manager_secret" "pepper" {
  secret_id = "account-code-pepper"
  replication {
    auto {}
  }
}

resource "google_secret_manager_secret_version" "pepper" {
  secret      = google_secret_manager_secret.pepper.id
  secret_data = var.pepper
}

resource "google_cloud_run_v2_service" "relay" {
  name     = "ouroboros-relay"
  location = var.region
  ingress  = "INGRESS_TRAFFIC_INTERNAL_LOAD_BALANCER"

  scaling {
    min_instance_count = 0
  }

  template {
    scaling {
      min_instance_count = 0
      max_instance_count = 1
    }
    service_account = var.runtime_service_account
    containers {
      image = var.image
      ports {
        container_port = 8080
      }
      env {
        name  = "SQLITE_PATH"
        value = "/tmp/relay.sqlite"
      }
      env {
        name  = "SNAPSHOT_PATH"
        value = "/tmp/relay.snapshot"
      }
    }
  }
}

resource "google_cloud_run_v2_service_iam_member" "invoker" {
  name     = google_cloud_run_v2_service.relay.name
  location = var.region
  role     = "roles/run.invoker"
  member   = "allUsers"
}

resource "google_cloud_tasks_queue" "wakes" {
  name     = "ouroboros-wakes"
  location = var.region
}

resource "google_compute_security_policy" "relay" {
  name = "ouroboros-relay"

  rule {
    action   = "rate_based_ban"
    priority = 1000
    match {
      versioned_expr = "SRC_IPS_V1"
      config {
        src_ip_ranges = ["*"]
      }
    }
    rate_limit_options {
      conform_action = "allow"
      exceed_action  = "deny(429)"
      enforce_on_key = "IP"
      rate_limit_threshold {
        count        = 120
        interval_sec = 60
      }
      ban_duration_sec = 300
    }
    description = "Per-IP rate limit in front of the relay."
  }

  rule {
    action   = "allow"
    priority = 2147483647
    match {
      versioned_expr = "SRC_IPS_V1"
      config {
        src_ip_ranges = ["*"]
      }
    }
    description = "Default allow."
  }
}

resource "google_compute_region_network_endpoint_group" "relay" {
  name                  = "ouroboros-relay"
  region                = var.region
  network_endpoint_type = "SERVERLESS"
  cloud_run {
    service = google_cloud_run_v2_service.relay.name
  }
}

resource "google_compute_backend_service" "relay" {
  name                  = "ouroboros-relay"
  protocol              = "HTTP"
  load_balancing_scheme = "EXTERNAL_MANAGED"
  security_policy       = google_compute_security_policy.relay.id

  backend {
    group = google_compute_region_network_endpoint_group.relay.id
  }
}

resource "google_compute_url_map" "relay" {
  name            = "ouroboros-relay"
  default_service = google_compute_backend_service.relay.id
}

resource "google_compute_managed_ssl_certificate" "relay" {
  name = "ouroboros-relay"
  managed {
    domains = ["relay.${var.project_id}.example.com"]
  }
}

resource "google_compute_target_https_proxy" "relay" {
  name             = "ouroboros-relay"
  url_map          = google_compute_url_map.relay.id
  ssl_certificates = [google_compute_managed_ssl_certificate.relay.id]
}

resource "google_compute_global_address" "relay" {
  name = "ouroboros-relay"
}

resource "google_compute_global_forwarding_rule" "relay" {
  name                  = "ouroboros-relay"
  ip_address            = google_compute_global_address.relay.id
  port_range            = "443"
  target                = google_compute_target_https_proxy.relay.id
  load_balancing_scheme = "EXTERNAL_MANAGED"
}

output "snapshot_bucket" {
  value = google_storage_bucket.snapshots.name
}

output "cloud_run_uri" {
  value = google_cloud_run_v2_service.relay.uri
}
