resource "google_project_service" "compute" {
  project            = var.project_id
  service            = "compute.googleapis.com"
  disable_on_destroy = false
}

resource "google_compute_address" "tuiper" {
  count  = var.create_static_ip ? 1 : 0
  name   = "${var.name_prefix}-ip"
  region = var.region

  depends_on = [google_project_service.compute]
}

resource "google_compute_instance" "tuiper" {
  name         = "${var.name_prefix}-vm"
  machine_type = var.machine_type
  zone         = var.zone

  tags = ["tuiper-server"]

  boot_disk {
    initialize_params {
      image = "ubuntu-os-cloud/ubuntu-2204-lts"
      size  = var.disk_size_gb
    }
  }

  network_interface {
    network = var.network
    access_config {
      nat_ip = var.create_static_ip ? google_compute_address.tuiper[0].address : null
    }
  }

  scheduling {
    automatic_restart = true
  }

  depends_on = [google_project_service.compute]
}

resource "google_compute_firewall" "tuiper_ws" {
  name    = "${var.name_prefix}-allow-ws"
  network = var.network

  description = "Allow WebSocket clients to reach tuiper-server"

  allow {
    protocol = "tcp"
    ports    = [tostring(var.server_port)]
  }

  source_ranges = var.firewall_source_ranges
  target_tags   = ["tuiper-server"]

  depends_on = [google_project_service.compute]
}
