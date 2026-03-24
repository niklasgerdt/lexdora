resource "kubernetes_config_map" "lexdora_config" {
  metadata {
    name      = "lexdora-config"
    namespace = kubernetes_namespace.lexdora.metadata[0].name
  }

  data = {
    "DATABASE_URL" = "postgres://lexdora:${var.db_password}@lexdora-db-service:5432/lexdora"
    "RUST_LOG"     = "info"
  }
}
