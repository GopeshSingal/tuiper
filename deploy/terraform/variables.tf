variable "project_id" {
  type        = string
  description = "GCP project ID where resources are created (gcloud projects list --format='value(projectId)')."
}

variable "region" {
  type        = string
  description = "Region for the static IP (if enabled) and regional resources."
  default     = "us-central1"
}

variable "zone" {
  type        = string
  description = "Zone for the Compute Engine VM."
  default     = "us-central1-a"
}

variable "name_prefix" {
  type        = string
  description = "Prefix for resource names (VM, firewall, address)."
  default     = "tuiper"
}

variable "machine_type" {
  type        = string
  description = "Compute Engine machine type."
  default     = "e2-micro"
}

variable "server_port" {
  type        = number
  description = "TCP port exposed for WebSocket traffic (must match tuiper-server PORT)."
  default     = 8080
}

variable "firewall_source_ranges" {
  type        = list(string)
  description = "CIDRs allowed to connect to server_port (e.g. [\"0.0.0.0/0\"] for the public internet)."
  default     = ["0.0.0.0/0"]
}

variable "network" {
  type        = string
  description = "VPC network name (default VPC is usually \"default\")."
  default     = "default"
}

variable "create_static_ip" {
  type        = bool
  description = "If true, reserve a regional static external IP and attach it to the VM."
  default     = false
}

variable "disk_size_gb" {
  type        = number
  description = "Boot disk size in GB."
  default     = 10
}
