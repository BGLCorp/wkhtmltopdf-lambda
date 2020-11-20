# What

`wkhtmltopdf-lambda` is a simple wrapper which runs [`wkhtmltopdf`](https://wkhtmltopdf.org/) as an AWS Lambda function.

# How

## Dependencies

1. [`rust`](https://www.rust-lang.org/), [`rustup`](https://rustup.rs/)
0. (Suggested) [musl libc](https://musl.libc.org/)
0. (Optional) [`just`](https://github.com/casey/just), [aws-cli](https://github.com/aws/aws-cli), [`jq`](https://github.com/stedolan/jq) and a few other common CLI tools

## Build & Deploy

1. `rustup target add x86_64-unknown-linux-musl`
0. Configure aws-cli credentials
0. Create an S3 bucket, e.g. `s3://wkhtmltopdf`
0. Create a role with CloudWatch and S3 permissions, e.g.
    ```json
    {
        "Version": "2012-10-17",
        "Statement": [
            {
                "Effect": "Allow",
                "Action": "logs:CreateLogGroup",
                "Resource": "arn:aws:logs:us-east-1:000000000000:*"
            },
            {
                "Effect": "Allow",
                "Action": [
                    "logs:CreateLogStream",
                    "logs:PutLogEvents"
                ],
                "Resource": [
                    "arn:aws:logs:us-east-1:000000000000:log-group:/aws/lambda/wkhtmltopdf-rust:*"
                ]
            },
            {
                "Effect": "Allow",
                "Action": [
                    "s3:GetObject"
                ],
                "Resource": "arn:aws:s3:::wkhtmltopdf/*"
            },
            {
                "Effect": "Allow",
                "Action": [
                    "s3:PutObject"
                ],
                "Resource": "arn:aws:s3:::wkhtmltopdf/*"
            }
        ]
    }
    ```
0. `just create-layer`
0. `env LAMBDA_ROLE="arn:aws:iam::000000000000:role/wkhtmltopdf" just create-function` (or use `.env` file)

Alternatively, instead of creating a layer, `just create-function true` can be used to bundle both the wrapper and `wkhtmltopdf` itself together.

## Test

`just test-function`

## Packaging

`just pack` (wrapper only) or `just pack true` (bundled `wkhtmltopdf`)

## [Serverless](https://www.serverless.com/)

See:

https://github.com/softprops/serverless-rust

https://github.com/vvilhonen/cargo-aws-lambda

## Targeting Glibc

See: https://github.com/softprops/lambda-rust
