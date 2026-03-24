variable "namespace_name" {
  description = "The name of the namespace to create"
  type        = string
  default     = "lexdora"
}

variable "app_version" {
  description = "The version of the application to deploy"
  type        = string
  default     = "latest"
}

variable "db_password" {
  description = "The password for the database"
  type        = string
  sensitive   = true
}

variable "kube_config_path" {
  description = "The path to the kubeconfig file"
  type        = string
  default     = "~/.kube/config"
}

variable "kube_context" {
  description = "The context to use in the kubeconfig file"
  type        = string
  default     = "minikube"
}
