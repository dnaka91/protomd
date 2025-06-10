repo := "dnaka91" / "protomd"

publish tag:
  git tag {{tag}}
  git cliff --current --strip all --output release.md

  jq --null-input \
    --arg tag_name "{{tag}}" \
    --rawfile body release.md \
    '$ARGS.named' > release.json

  curl -iX POST \
    -H "Accept: application/json" \
    -H "Authorization: token $FORGE_TOKEN" \
    -H "Content-Type: application/json" \
    -d @release.json \
    https://forge.dnaka91.rocks/api/v1/repos/{{repo}}/releases

build-assets:
  cargo zigbuild --release --target aarch64-unknown-linux-musl
  cargo zigbuild --release --target x86_64-unknown-linux-musl
  cargo zigbuild --release --target aarch64-apple-darwin
  cargo zigbuild --release --target x86_64-apple-darwin
  cargo zigbuild --release --target aarch64-pc-windows-gnullvm
  cargo zigbuild --release --target x86_64-pc-windows-gnullvm

build-asset target:
  cargo zigbuild --release --target {{target}}
