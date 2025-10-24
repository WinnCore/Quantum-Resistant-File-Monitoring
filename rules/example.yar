rule DemoARM64Malware
{
    meta:
        description = "Example rule for ARM64 suspicious ELF"
        author = "CharmedWOA Security"
        reference = "https://charmedwoa.example"
        ab_bucket = "baseline"
    strings:
        $elf = { 7F 45 4C 46 02 01 01 }
        $packed = /UPX[0-9]{2}/ nocase
    condition:
        $elf and $packed
}
