resource "kubernetes_deployment" "database" {
  metadata {
    name      = "lexdora-db"
    namespace = kubernetes_namespace.lexdora.metadata[0].name
    labels = {
      app = "lexdora-db"
    }
  }

  spec {
    replicas = 1
    selector {
      match_labels = {
        app = "lexdora-db"
      }
    }

    template {
      metadata {
        labels = {
          app = "lexdora-db"
        }
      }

      spec {
        container {
          name  = "postgres"
          image = "postgres:15-alpine"

          port {
            container_port = 5432
          }

          env {
            name  = "POSTGRES_DB"
            value = "lexdora"
          }
          env {
            name  = "POSTGRES_USER"
            value = "lexdora"
          }
          env {
            name  = "POSTGRES_PASSWORD"
            value = var.db_password
          }

          resources {
            requests = {
              cpu    = "100m"
              memory = "256Mi"
            }
            limits = {
              cpu    = "500m"
              memory = "512Mi"
            }
          }
        }
      }
    }
  }
}

resource "kubernetes_service" "database" {
  metadata {
    name      = "lexdora-db-service"
    namespace = kubernetes_namespace.lexdora.metadata[0].name
  }

  spec {
    selector = {
      app = "lexdora-db"
    }

    port {
      port        = 5432
      target_port = 5432
    }

    type = "ClusterIP"
  }
}
