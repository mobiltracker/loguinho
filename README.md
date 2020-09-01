# Loguinho

## Cli helper for cloudwatch logs

### Aws Credentials setup: 
This project uses [rusoto_credential::ChainProvider](https://rusoto.github.io/rusoto/rusoto_credential/struct.ChainProvider.html) to provide aws credentials

The following sources are checked in order for credentials when calling credentials:
- Environment variables: AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY
- credential_process command in the AWS config file, usually located at ~/.aws/config.
- AWS credentials file using [loguinho] profile. Usually located at ~/.aws/credentials.
- AWS credentials file using [default] profile. Usually located at ~/.aws/credentials.
- IAM instance profile. Will only work if running on an EC2 instance with an instance profile/role.




## Commands:

### Watch:

- loguinho watch [filter] : tails cloudwatch log events from log groups matching [filter] parameter
