// EICAR test file detection
rule EICAR_Test_File
{
    meta:
        description = "Detects EICAR antivirus test file"
        author = "Zachary Winn <zw@winncore.com>"
        reference = "https://www.eicar.org/download-anti-malware-testfile/"
        severity = "high"
        ab_bucket = "baseline"

    strings:
        $eicar = "X5O!P%@AP[4\\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*"

    condition:
        $eicar
}

// UPX packed executable detection
rule UPX_Packed_ARM64_ELF
{
    meta:
        description = "Detects UPX-packed ARM64 ELF binaries"
        author = "Zachary Winn <zw@winncore.com>"
        reference = "https://upx.github.io/"
        severity = "medium"
        ab_bucket = "baseline"

    strings:
        $elf_magic = { 7F 45 4C 46 02 01 01 }  // ELF64 header
        $upx1 = "UPX!"
        $upx2 = /UPX[0-9]{1,2}/
        $upx_section = ".upx"

    condition:
        $elf_magic at 0 and any of ($upx*)
}

// Suspicious ARM64 ELF with excessive syscalls
rule Suspicious_ARM64_Syscall_Heavy
{
    meta:
        description = "ARM64 ELF with unusual syscall density"
        author = "Zachary Winn <zw@winncore.com>"
        severity = "medium"
        ab_bucket = "heuristic"

    strings:
        $elf_magic = { 7F 45 4C 46 02 01 01 }
        $arm64_marker = { B7 ?? ?? ?? ?? }  // ARM64 architecture

        // Common syscall patterns (SVC instruction on ARM64)
        $svc = { D4 00 00 01 }  // SVC #0

    condition:
        $elf_magic at 0 and
        $arm64_marker and
        #svc > 50  // More than 50 direct syscalls
}

// Reverse shell indicators
rule Reverse_Shell_Indicators
{
    meta:
        description = "Detects common reverse shell patterns"
        author = "Zachary Winn <zw@winncore.com>"
        severity = "high"
        ab_bucket = "baseline"

    strings:
        $socket_connect = "connect"
        $dup2 = "dup2"
        $execve = "execve"
        $shell1 = "/bin/sh"
        $shell2 = "/bin/bash"

        // Network indicators
        $sockaddr = "sockaddr"
        $inet = "AF_INET"

    condition:
        4 of them
}

// Crypto mining indicators
rule Crypto_Miner_Indicators
{
    meta:
        description = "Detects cryptocurrency mining patterns"
        author = "Zachary Winn <zw@winncore.com>"
        severity = "medium"
        ab_bucket = "heuristic"

    strings:
        $xmrig = "xmrig" nocase
        $monero = "monero" nocase
        $stratum = "stratum+" nocase
        $mining_pool1 = "pool.minexmr" nocase
        $mining_pool2 = "pool.supportxmr" nocase
        $mining_pool3 = "hashvault" nocase

        // Mining config indicators
        $donate_level = "donate-level"
        $algo = "rx/0" nocase  // RandomX algorithm
        $cpu_affinity = "cpu-affinity"

    condition:
        3 of them
}

// Suspicious file modification patterns
rule Mass_File_Encryptor
{
    meta:
        description = "Detects ransomware-like file encryption patterns"
        author = "Zachary Winn <zw@winncore.com>"
        severity = "critical"
        ab_bucket = "baseline"

    strings:
        $crypto1 = "AES" nocase
        $crypto2 = "RSA" nocase
        $crypto3 = "ChaCha20"

        $file_ops1 = "rename"
        $file_ops2 = "unlink"
        $file_ops3 = "openat"

        $ransom_ext1 = ".encrypted"
        $ransom_ext2 = ".locked"
        $ransom_ext3 = ".crypt"

        $recursion = "readdir"

    condition:
        any of ($crypto*) and
        2 of ($file_ops*) and
        $recursion
}
