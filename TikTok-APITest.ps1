

<#
This file previously contained hard-coded/committed client credentials — they have been removed.

Use environment variables to store secrets securely and avoid committing them to source control.

Environment variables used:
- `TIKTOK_CLIENT_KEY`
- `TIKTOK_CLIENT_SECRET`

Example (current session):
$env:TIKTOK_CLIENT_KEY = 'your_client_key'
$env:TIKTOK_CLIENT_SECRET = 'your_client_secret'

Persist for the current user:
[Environment]::SetEnvironmentVariable('TIKTOK_CLIENT_KEY','your_client_key','User')
[Environment]::SetEnvironmentVariable('TIKTOK_CLIENT_SECRET','your_client_secret','User')

Replace the placeholder values above with your real credentials obtained from TikTok's developer portal.
#>

# Parameters for OAuth and endpoints
param(
	[string]$RedirectUri = "http://localhost:8080/callback/",
	[string]$AuthEndpoint = "https://open.tiktokapis.com/platform/oauth/connect/",
	[string]$TokenEndpoint = "https://open.tiktokapis.com/oauth/access_token/",
	[string]$CommentsEndpoint = "https://open.tiktokapis.com/v1/comments",
	[string]$Scopes = "user.comments.read",
	[switch]$UseBrowser = $true
)

# Read credentials from environment first
$clientKey = $env:TIKTOK_CLIENT_KEY
$clientSecret = $env:TIKTOK_CLIENT_SECRET


# If not found in env, try PowerShell SecretManagement vault (recommended)
if (-not $clientKey -or -not $clientSecret) {
	if (Get-Module -ListAvailable -Name Microsoft.PowerShell.SecretManagement) {
		try {
			$maybeKey = Get-Secret -Name 'TIKTOK_CLIENT_KEY' -ErrorAction Stop
			if ($maybeKey) { $clientKey = $maybeKey }
		} catch { }

		try {
			$maybeSecret = Get-Secret -Name 'TIKTOK_CLIENT_SECRET' -ErrorAction Stop
			if ($maybeSecret) {
				$clientSecret = [Runtime.InteropServices.Marshal]::PtrToStringAuto(
					[Runtime.InteropServices.Marshal]::SecureStringToBSTR($maybeSecret)
				)
			}
		} catch { }
	}
}

if (-not $clientKey -or -not $clientSecret) {
	Write-Host "Error: TIKTOK_CLIENT_KEY and/or TIKTOK_CLIENT_SECRET not found." -ForegroundColor Red
	Write-Host "Options to provide credentials:" -ForegroundColor Cyan
	Write-Host "1) Set for current session (temporary):" -ForegroundColor Cyan
	Write-Host '   $env:TIKTOK_CLIENT_KEY = ''your_client_key''' -ForegroundColor Cyan
	Write-Host '   $env:TIKTOK_CLIENT_SECRET = ''your_client_secret''' -ForegroundColor Cyan
	Write-Host "2) Persist for user (Windows):" -ForegroundColor Cyan
	Write-Host "   [Environment]::SetEnvironmentVariable('TIKTOK_CLIENT_KEY','your_client_key','User')" -ForegroundColor Cyan
	Write-Host "   [Environment]::SetEnvironmentVariable('TIKTOK_CLIENT_SECRET','your_client_secret','User')" -ForegroundColor Cyan
	Write-Host "3) Use PowerShell SecretManagement (recommended):" -ForegroundColor Cyan
	Write-Host "   Install-Module Microsoft.PowerShell.SecretManagement,Microsoft.PowerShell.SecretStore -Scope CurrentUser" -ForegroundColor Cyan
	Write-Host "   Register-SecretVault -Name SecretStore -ModuleName Microsoft.PowerShell.SecretStore -DefaultVault" -ForegroundColor Cyan
	Write-Host "   Set-Secret -Name TIKTOK_CLIENT_KEY -Secret 'your_client_key'" -ForegroundColor Cyan
	Write-Host "   Set-Secret -Name TIKTOK_CLIENT_SECRET -Secret 'your_client_secret'" -ForegroundColor Cyan
	return
}

Write-Host "Credentials loaded." -ForegroundColor Green

# OAuth and comment-fetching implementation (parameters defined above)

function Start-LocalCallbackListener {
	param([int]$Port = 8080)
	$listener = New-Object System.Net.HttpListener
	$prefix = "http://localhost:$Port/"
	$listener.Prefixes.Add($prefix)
	$listener.Start()
	return $listener
}

function Wait-For-AuthCode {
	param(
		$listener,
		[int]$TimeoutSeconds = 120
	)

	Write-Host "Waiting for OAuth callback..." -ForegroundColor Cyan
	$task = $listener.GetContextAsync()
	$completed = $task.Wait([TimeSpan]::FromSeconds($TimeoutSeconds))

	if ($completed) {
		$context = $task.Result
		$request = $context.Request
		$response = $context.Response
		$qs = $request.QueryString
		$code = $qs['code']
		$buffer = [System.Text.Encoding]::UTF8.GetBytes("You may close this window and return to the script.")
		$response.ContentLength64 = $buffer.Length
		$response.OutputStream.Write($buffer,0,$buffer.Length)
		$response.OutputStream.Close()
		$listener.Stop()
		return $code
	}

	Write-Host "No callback received within $TimeoutSeconds seconds." -ForegroundColor Yellow
	Write-Host "Paste either the full callback URL (http://localhost:8080/callback/?code=...) or just the code value:" -ForegroundColor Cyan
	$manual = Read-Host "Callback URL or code"
	$listener.Stop()

	if ([string]::IsNullOrWhiteSpace($manual)) {
		return $null
	}

	if ($manual -match '[\?&]code=([^&]+)') {
		return [uri]::UnescapeDataString($Matches[1])
	}

	return $manual.Trim()
}

function Exchange-AuthCode-For-Token {
	param($code, $clientId, $clientSecret, $redirectUri, $tokenEndpoint)
	$body = @{
		client_id     = $clientId
		client_secret = $clientSecret
		code          = $code
		grant_type    = 'authorization_code'
		redirect_uri  = $redirectUri
	}
	try {
		$resp = Invoke-RestMethod -Uri $tokenEndpoint -Method Post -Body $body -ContentType 'application/x-www-form-urlencoded'
		return $resp
	} catch {
		Write-Host "Token exchange failed:`n$($_.Exception.Message)" -ForegroundColor Red
		return $null
	}
}

function Get-Comments {
	param($commentsEndpoint, $accessToken, $queryParams)
	$headers = @{ Authorization = "Bearer $accessToken" }
	$uri = $commentsEndpoint
	if ($queryParams) { $uri = "$uri?$queryParams" }
	try {
		$resp = Invoke-RestMethod -Uri $uri -Headers $headers -Method Get
		return $resp
	} catch {
		Write-Host "Failed to fetch comments:`n$($_.Exception.Message)" -ForegroundColor Red
		return $null
	}
}

# Build auth URL and perform OAuth flow
$clientId = $clientKey
$clientSecretPlain = $clientSecret

if (-not $clientId -or -not $clientSecretPlain) {
	Write-Host "Missing client credentials; cannot start OAuth flow." -ForegroundColor Red
	return
}

$state = [Guid]::NewGuid().ToString()
$authUrl = "${AuthEndpoint}?response_type=code&client_id=$([uri]::EscapeDataString($clientId))&redirect_uri=$([uri]::EscapeDataString($RedirectUri))&scope=$([uri]::EscapeDataString($Scopes))&state=$state"

Write-Host "Open the following URL to authorize the app:" -ForegroundColor Cyan
Write-Host $authUrl -ForegroundColor Yellow

if ($UseBrowser) {
	try {
		Start-Process -FilePath $authUrl -ErrorAction Stop
	} catch {
		# Fallback to cmd 'start' which reliably opens the default browser
		Start-Process -FilePath "cmd.exe" -ArgumentList "/c","start","",$authUrl
	}
}

# Start listener and wait for code
$uriObj = [uri]$RedirectUri
$port = $uriObj.Port
$listener = Start-LocalCallbackListener -Port $port
$code = Wait-For-AuthCode -listener $listener

if (-not $code) {
	Write-Host "Authorization code not received." -ForegroundColor Red
	return
}

Write-Host "Received authorization code." -ForegroundColor Green

$tokenResp = Exchange-AuthCode-For-Token -code $code -clientId $clientId -clientSecret $clientSecretPlain -redirectUri $RedirectUri -tokenEndpoint $TokenEndpoint

if (-not $tokenResp -or -not $tokenResp.access_token) {
	Write-Host "Failed to obtain access token." -ForegroundColor Red
	return
}

$accessToken = $tokenResp.access_token
Write-Host "Access token obtained." -ForegroundColor Green

# Example: fetch comments. The exact query params depend on the API; replace as needed.
$query = "username=bagofpipes&limit=50"
$comments = Get-Comments -commentsEndpoint $CommentsEndpoint -accessToken $accessToken -queryParams $query
if ($comments) {
	Write-Host "Comments fetched:" -ForegroundColor Green
	$comments | ConvertTo-Json -Depth 4
}

Write-Host "Done." -ForegroundColor Green