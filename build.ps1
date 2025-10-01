# Build the binary first
$env:RUSTFLAGS="-C strip=symbols -C opt-level=z -C link-arg=-s";
cargo build --release

# now move the binary to destination folder
$dest = "C:\src\up_bins"
if (!(Test-Path -Path $dest)) {
    New-Item -ItemType Directory -Path $dest
}
Copy-Item -Path ".\target\release\crun.exe" -Destination $dest -Force

