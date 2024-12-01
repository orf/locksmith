target "default" {
  name = "${version}"
  context    = "docker-postgres/"
  dockerfile = "${version}/bookworm/Dockerfile"
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
  tags = ["pg:${version}"]
}
