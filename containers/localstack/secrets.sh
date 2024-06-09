#!/bin/bash

# This script is used only in local environment to setup the secret manager

export AWS_SECRET_ACCESS_KEY=$AWS_SECRET_ACCESS_KEY
export AWS_ACCESS_KEY_ID=$AWS_ACCESS_KEY_ID
export AWS_REGION=$AWS_REGION
export ENDPOINT=$ENDPOINT

sleep 5  # Wait for services to start

echo "Seeding Secret Manager..."

aws secretsmanager create-secret --endpoint-url $ENDPOINT --name secret-1 --secret-string 84cd01f7f3e07756be8c3e133275616308921356f30fe0df63cd56fdf26da8ae
aws secretsmanager create-secret --endpoint-url $ENDPOINT --name secret-2 --secret-string 4ce6a2e359976bfd186eb24c19fe0223a241add277b649bb9e5e8464ee36a9d7
aws secretsmanager create-secret --endpoint-url $ENDPOINT --name secret-3 --secret-string 498181c80a3ecd8c2a9a05c5570f62990e5aae0e2d25743178300d7f5e9bf9d2
# The secret is "54a5d2d0ee46c477f4a5b4c2570099ac91aa98dcadd033c460f46853fc362f9" in base64
aws secretsmanager create-secret --endpoint-url $ENDPOINT --name secret-4 --secret-binary NTRhNWQyZDBlZTQ2YzQ3N2Y0YTViNGMyNTcwMDk5YWM5MWFhOThkY2FkZDAzM2M0NjBmNDY4NTNmYzM2MmY5ZA==
# The secret is "9c98e5d1cd7582e11a32646216a64adf62a8c484901aa5c9fd722fc7465a19f0" in base64
aws secretsmanager create-secret --endpoint-url $ENDPOINT --name secret-5 --secret-binary OWM5OGU1ZDFjZDc1ODJlMTFhMzI2NDYyMTZhNjRhZGY2MmE4YzQ4NDkwMWFhNWM5ZmQ3MjJmYzc0NjVhMTlmMA==
# The secret is "0ae4b3a49454a3a8b9f7c1eed386c6c762283023d725d9591521718dfe9764a1" in base64
aws secretsmanager create-secret --endpoint-url $ENDPOINT --name secret-6 --secret-binary MGFlNGIzYTQ5NDU0YTNhOGI5ZjdjMWVlZDM4NmM2Yzc2MjI4MzAyM2Q3MjVkOTU5MTUyMTcxOGRmZTk3NjRhMQ==

# Secrets with different versions

aws secretsmanager create-secret --endpoint-url $ENDPOINT --name versioned-secret --secret-string 51cc0c173419b77cedcaf322411262018cd012a95920a3c4d7ae577ff76c4b92
aws secretsmanager update-secret --endpoint-url $ENDPOINT --secret-id versioned-secret --secret-string a329ca5df23159a7fa6400f919193fb02b59bc9cdc7d6527f1ca2cb7ed668121

echo "Secret Manager seeded!"
