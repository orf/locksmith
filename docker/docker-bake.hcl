variable "IMAGE_REPOSITORY" {
  default = "local-pg"
}

target "default" {
  name = "${version}"
  context    = "docker-postgres/"
  dockerfile = "${version}/bookworm/Dockerfile"
  attest = [
    "type=provenance,mode=max",
    "type=sbom"
  ]
  platforms = ["linux/amd64", "linux/arm64"]
  matrix = {
    version = [
      "12",
      "13",
      "14",
      "15",
      "16",
      "17",
    ]
  }
  tags = ["${IMAGE_REPOSITORY}:${version}"]
}
