# Interrupt Assignment Process

1. Parse ACPI tables
2. Identify Interrupt Source Overrides (ISOs)
3. Enumerate PCI bus
    1. Find devices
    2. Get _CRS from ACPI AML
    3. Look for ISO, if it exists, use that as the GSI
    4. Assign GSI IRQ if it is free or sharable
    5. If it is not free or sharable, look at _PRS
4. Load PCI device drivers
5. Driver requests MSI
    1. Driver asks kernel for MSI interupt
    2. Kernel frees GSI
6. Driver does not request MSI
    1. Driver uses GSI