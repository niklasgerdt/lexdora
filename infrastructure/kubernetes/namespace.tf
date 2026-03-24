resource "kubernetes_namespace" "lexdora" {
  metadata {
    name = var.namespace_name
  }
}
