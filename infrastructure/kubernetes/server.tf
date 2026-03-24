resource "kubernetes_deployment" "server" {
  metadata {
    name      = "lexdora-server"
    namespace = kubernetes_namespace.lexdora.metadata[0].name
    labels = {
      app = "lexdora-server"
    }
  }

  spec {
    replicas = 1
    selector {
      match_labels = {
        app = "lexdora-server"
      }
    }

    template {
      metadata {
        labels = {
          app = "lexdora-server"
        }
      }

      spec {
        container {
          name              = "server"
          image             = "lexdora-server:${var.app_version}"
          image_pull_policy = "IfNotPresent"

          port {
            container_port = 8080
          }

          env_from {
            config_map_ref {
              name = kubernetes_config_map.lexdora_config.metadata[0].name
            }
          }

          resources {
            requests = {
              cpu    = "250m"
              memory = "512Mi"
            }
            limits = {
              cpu    = "1000m"
              memory = "1Gi"
            }
          }

          liveness_probe {
            http_get {
              path = "/healthz"
              port = 8080
            }
            initial_delay_seconds = 60
            period_seconds        = 10
          }

          readiness_probe {
            http_get {
              path = "/healthz"
              port = 8080
            }
            initial_delay_seconds = 30
            period_seconds        = 5
          }
        }
      }
    }
  }
}

resource "kubernetes_service" "server" {
  metadata {
    name      = "lexdora-server-service"
    namespace = kubernetes_namespace.lexdora.metadata[0].name
  }

  spec {
    selector = {
      app = "lexdora-server"
    }

    port {
      port        = 80
      target_port = 8080
    }

    type = "NodePort" # For local access
  }
}
