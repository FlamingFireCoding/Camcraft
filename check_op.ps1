$data = Get-Content 'examples/first-continuous-4axis-part.camproj' -Raw | ConvertFrom-Json
$op = $data.entities | Where-Object { $_.kind -eq 'operation' }
$op | Format-List *
