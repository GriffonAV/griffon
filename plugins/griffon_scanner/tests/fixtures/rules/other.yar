// Minimal offline ruleset for unit tests.
// The file stem ("other") maps to ThreatCategory::Other.
// Matches the EICAR test file without embedding the full signature here,
// so this file itself is not flagged by antivirus software.

rule Test_EICAR
{
    strings:
        $marker = "EICAR-STANDARD-ANTIVIRUS-TEST-FILE"

    condition:
        $marker
}
