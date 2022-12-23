terraform {
  required_providers {
    docker = {
      source  = "kreuzwerker/docker"
      version = "2.24.0"
    }
  }
}

data "aws_caller_identity" "current" {}

data "aws_region" "current" {}

# Image
# ‾‾‾‾‾

locals {
  image_registry = "${data.aws_caller_identity.current.account_id}.dkr.ecr.${data.aws_region.current.id}.amazonaws.com"
}

data "aws_ecr_authorization_token" "token" {}

provider "docker" {
  registry_auth {
    address  = local.image_registry
    username = data.aws_ecr_authorization_token.token.user_name
    password = data.aws_ecr_authorization_token.token.password
  }
}

data "local_file" "dockerfile" {
  filename = abspath("${path.module}/image/Dockerfile")
}

module "docker_image" {
  source = "terraform-aws-modules/lambda/aws//modules/docker-build"

  create_ecr_repo = true
  ecr_repo        = "${var.name}-lambda"
  # Pulls tag from Dockerfile
  image_tag   = regex("(?m:^FROM.+:(.+)$)", data.local_file.dockerfile.content)[0]
  source_path = abspath("${path.module}/image")

  ecr_repo_lifecycle_policy = jsonencode({
    "rules" : [
      {
        "rulePriority" : 1,
        "description" : "Expire old versions",
        "selection" : {
          "tagStatus" : "any",
          "countType" : "imageCountMoreThan",
          "countNumber" : 2,
        },
        "action" : {
          "type" : "expire"
        }
      }
    ]
  })
}

# Lambda
# ‾‾‾‾‾‾

module "lambda_function_container_image" {
  source = "terraform-aws-modules/lambda/aws"

  function_name = var.name

  create_lambda_function_url = true
  timeout                    = 30

  create_package = false
  package_type   = "Image"
  image_uri      = module.docker_image.image_uri

  environment_variables = {
    "PLATE_API_JWT_SECRET" = var.jwt_secret
    "RUST_LOG" = "debug"
  }

}
