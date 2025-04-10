# Build documentation script for gooty-proxy
# This script runs cargo doc with private items and copies the output to the workspace root

Write-Host "Building documentation with cargo doc..." -ForegroundColor Cyan

# Run cargo doc with options to include private items and not build dependencies
cargo doc --no-deps --document-private-items

if ($LASTEXITCODE -ne 0) {
    Write-Host "Failed to build documentation. Exiting." -ForegroundColor Red
    exit $LASTEXITCODE
}

Write-Host "Documentation built successfully." -ForegroundColor Green

# Define the source and destination paths
$sourceDir = "./target/doc"
$destDir = "./docs"

# Check if source directory exists
if (-not (Test-Path $sourceDir)) {
    Write-Host "Documentation directory $sourceDir not found. Exiting." -ForegroundColor Red
    exit 1
}

# Create destination directory if it doesn't exist
if (-not (Test-Path $destDir)) {
    Write-Host "Creating docs directory..." -ForegroundColor Yellow
    New-Item -ItemType Directory -Path $destDir | Out-Null
}
else {
    Write-Host "Clearing existing docs directory..." -ForegroundColor Yellow
    Remove-Item -Path "$destDir/*" -Recurse -Force
}

# Copy documentation files to workspace root
Write-Host "Copying documentation to $destDir..." -ForegroundColor Cyan
Copy-Item -Path "$sourceDir/*" -Destination $destDir -Recurse

Write-Host "Documentation copied to $destDir successfully." -ForegroundColor Green
Write-Host "You can view the documentation by opening ./docs/gooty_proxy/index.html in your browser." -ForegroundColor Cyan
