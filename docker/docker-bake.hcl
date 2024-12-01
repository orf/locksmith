target "default" {
  context = "docker-postgres/"
  dockerfile = "12/bookworm/Dockerfile"
  # contexts = {
  # blob/master/16/bookworm/Dockerfile
  # foo = "https://github.com/docker-library/postgres.git"
  # }

  # args = {
  #   NODE_VERSION = "22"
  # }
  tags = ["foo"]
}
