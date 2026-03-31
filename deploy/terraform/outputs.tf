output "external_ip" {
  description = "External IP for WS_URL (e.g. ws://IP:8080/ws)."
  value       = google_compute_instance.tuiper.network_interface[0].access_config[0].nat_ip
}

output "instance_name" {
  description = "Name of the Compute Engine VM."
  value       = google_compute_instance.tuiper.name
}

output "zone" {
  description = "Zone the VM runs in."
  value       = var.zone
}

output "ws_url_example" {
  description = "Example client WS_URL using external_ip and server_port."
  value       = "ws://${google_compute_instance.tuiper.network_interface[0].access_config[0].nat_ip}:${var.server_port}/ws"
}
