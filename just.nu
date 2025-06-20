#!/usr/bin/env nu
const repo = "dnaka91/protomd"
const fj_base_url = "https://forge.dnaka91.rocks/api/v1"
const gh_base_url = "https://api.github.com"

# Converts a .env file into a record
#
# From: https://github.com/nushell/nu_scripts/blob/main/modules/formats/from-env.nu
def "from env" []: string -> record {
  lines
  | split column '#' # remove comments
  | get column1
  | parse "{key}={value}"
  | update value {
    str trim                      # Trim whitespace between value and inline comments
    | str trim -c '"'             # unquote double-quoted values
    | str trim -c "'"             # unquote single-quoted values
    | str replace -a "\\n" "\n"   # replace `\n` with newline char
    | str replace -a "\\r" "\r"   # replace `\r` with carriage return
    | str replace -a "\\t" "\t"   # replace `\t` with tab
  }
  | transpose -r -d
}

if (".env" | path exists) {
  open .env | load-env
}

def main [] {}

# Publish a new version to the Forge(s)
def "main publish" [tag: string] {
  let files = build-assets $tag

  create-tag $tag

  let id = fj create-release $tag
  $files | each { fj upload-asset $id }

  let id = gh create-release $tag
  $files | each { gh upload-asset $id }

  print $"(ansi gb)==>(ansi yb) successfully released (ansi bb)($tag)(ansi reset)"
}

def build-assets [tag: string]: nothing -> list<path> {
  $env.RUSTFLAGS = (
    [
      $"(cargo metadata --format-version 1 | from json | get workspace_root)=src"
      $"($env.CARGO_HOME? | (default $nu.home-path | path join .cargo))=cargo"
      $"($env.RUSTUP_HOME? | (default $nu.home-path | path join .rustup))=rust"
    ]
    | each {["--remap-path-prefix" $in]}
    | flatten
    | str join ' '
  )

  [aarch64 x86_64]
  | each { |arch| [
      $"($arch)-unknown-linux-musl"
      $"($arch)-apple-darwin"
      $"($arch)-pc-windows-gnullvm"
  ] }
  | flatten
  | each { build-asset $tag }
  | flatten
}

def build-asset [tag: string]: string -> list<path> {
  let target = $in
  let dir = "dist"
  let base_name = $"protomd-($tag)-($target)"
  let tar_file  = $"($base_name).tar"
  let zst_file  = $"($tar_file).zst"
  let chk_file = $"($base_name).sha512"

  print $"(ansi gb)==>(ansi w) building target (ansi bb)($target)(ansi reset)"

  cargo zigbuild --release --target $target

  mkdir $dir

  (
    tar --create
      --file ($dir | path join $tar_file)
      --directory $"target/($target)/release"
      $"protomd(if ($target | str contains "windows") { ".exe" })"
  )

  cd $dir
  zstdmt --rm --force $tar_file -o $zst_file
  sha512sum $zst_file | save -f $chk_file

  [$zst_file $chk_file] | each {|f| $dir | path join $f }
}

def create-tag [ tag:string]: nothing -> nothing {
  print $"(ansi gb)==>(ansi w) creating tag (ansi bb)($tag)(ansi reset)"
  git tag $tag
  git push --tags
}

let fj_headers = {
  authorization: $"token ($env.FORGE_TOKEN)"
}

def "fj create-release" [tag: string]: nothing -> int {
  let url = $"($fj_base_url)/repos/($repo)/releases"

  print $"(ansi gb)==>(ansi w) creating release on (ansi bb)forgejo(ansi reset)"

  (
    http post
      --headers $fj_headers
      --content-type "application/json"
      $url {
        tag_name: $tag
        body: (git cliff --current --strip all)
        draft: false
      }
  )
  | get id
}

def "fj upload-asset" [id: int]: path -> nothing {
  let query = $in | path basename | {name: $in} | url build-query
  let url = $"($fj_base_url)/repos/($repo)/releases/($id)/assets?($query)"

  print $"  (ansi gb)==>(ansi w) uploading (ansi bb)($in)(ansi reset)"

  (
    http post
      --headers $fj_headers
      --content-type "multipart/form-data"
      $url {
        attachment: (open -r $in | into binary)
      }
  )
}

let gh_headers = {
  accept: "application/vnd.github+json"
  authorization: $"Bearer ($env.GITHUB_TOKEN)"
  x-github-api-version: "2022-11-28"
}

def "gh create-release" [tag: string]: nothing -> int {
  let url = $"($gh_base_url)/repos/($repo)/releases"

  print $"(ansi gb)==>(ansi w) creating release on (ansi bb)github(ansi reset)"

  (
    http post
      --headers $gh_headers
      --content-type "application/json"
      $url {
        tag_name: $tag
        body: (git cliff --current --strip all)
        draft: false
      }
  )
  | get id
}

def "gh upload-asset" [id: int]: path -> nothing {
  let query = $in | path basename | {name: $in} | url build-query
  let url = $"https://uploads.github.com/repos/($repo)/releases/($id)/assets?($query)"

  print $"  (ansi gb)==>(ansi w) uploading (ansi bb)($in)(ansi reset)"

  (
    http post
      --headers $gh_headers
      --content-type "application/zstd"
      $url
      (open -r $in | into binary)
  )
}
