pub struct Name {
    name: u64,
}

impl Name {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        arbitrary_address_capable: bool,
        industry_group: u8,
        vehicle_system_instance: u8,
        vehicle_system: u8,
        function: u8,
        function_instance: u8,
        ecu_instance: u8,
        manufacturer_code: u16,
        identity_number: u32,
    ) -> Self {
        let mut name: u64 = 0;

        // Arbitrary address capable - 1 bit
        if arbitrary_address_capable {
            name |= 0x8000000000000000;
        }

        // Industry group - 3 bits
        name |= (industry_group as u64) << 60;

        // Vehicle system instance - 4 bits
        name |= (vehicle_system_instance as u64) << 56;

        // Vehicle system - 7 bits
        name |= (vehicle_system as u64) << 49;

        // Reserved bit

        // Function - 8 bits
        name |= (function as u64) << 40;

        // Function instance - 5 bits
        name |= (function_instance as u64) << 35;

        // ECU instance - 3 bits
        name |= (ecu_instance as u64) << 32;

        // Manufacturer code - 11 bits
        name |= (manufacturer_code as u64) << 21;

        // Identity number - 21 bits
        name |= identity_number as u64;

        Name { name }
    }

    // Arbitrary address capable - 1 bit
    pub fn arbitrary_address_capable(&self) -> bool {
        self.name & 0x8000000000000000 > 0
    }

    // Industry group - 3 bits
    pub fn industry_group(&self) -> u8 {
        ((self.name >> 60) & 0x07) as u8
    }

    // Vehicle system instance - 4 bits
    pub fn vehicle_system_instance(&self) -> u8 {
        ((self.name >> 56) & 0x0f) as u8
    }

    // Vehicle system - 7 bits
    pub fn vehicle_system(&self) -> u8 {
        ((self.name >> 49) & 0x7f) as u8
    }

    // Reserved bit

    // Function - 8 bits
    pub fn function(&self) -> u8 {
        ((self.name >> 40) & 0xff) as u8
    }

    // Function instance - 5 bits
    pub fn function_instance(&self) -> u8 {
        ((self.name >> 35) & 0x1f) as u8
    }

    // ECU instance - 3 bits
    pub fn ecu_instance(&self) -> u8 {
        ((self.name >> 32) & 0x07) as u8
    }

    // Manufacturer code - 11 bits
    pub fn manufacturer_code(&self) -> u16 {
        ((self.name >> 21) & 0x07ff) as u16
    }

    // Identity number - 21 bits
    pub fn identity_number(&self) -> u32 {
        (self.name & 0x1fffff) as u32
    }

    pub fn value(&self) -> u64 {
        self.name
    }
}

#[cfg(test)]
mod tests {
    use crate::Name;

    #[test]
    fn name_new() {
        struct TestCase {
            arbitrary_address_capable: bool,
            industry_group: u8,
            vehicle_system_instance: u8,
            vehicle_system: u8,
            function: u8,
            function_instance: u8,
            ecu_instance: u8,
            manufacturer_code: u16,
            identity_number: u32,
        }
        let test_cases = [
            TestCase {
                arbitrary_address_capable: true, // 1 bit
                industry_group: 0x02,            // 3 bits
                vehicle_system_instance: 0x05,   // 4 bits
                vehicle_system: 0x55,            // 7 bits
                function: 0x55,                  // 8 bits
                function_instance: 0x15,         // 5 bits
                ecu_instance: 0x05,              // 3 bits
                manufacturer_code: 0x0555,       // 11 bits
                identity_number: 0x00155555,     // 21 bits
            },
            TestCase {
                arbitrary_address_capable: true, // 1 bit
                industry_group: 0x02,            // 3 bits
                vehicle_system_instance: 0x0a,   // 4 bits
                vehicle_system: 0x55,            // 7 bits
                function: 0xaa,                  // 8 bits
                function_instance: 0x15,         // 5 bits
                ecu_instance: 0x02,              // 3 bits
                manufacturer_code: 0x0555,       // 11 bits
                identity_number: 0x000aaaaa,     // 21 bits
            },
            TestCase {
                arbitrary_address_capable: false, // 1 bit
                industry_group: 0x05,             // 3 bits
                vehicle_system_instance: 0x05,    // 4 bits
                vehicle_system: 0x2a,             // 7 bits
                function: 0x55,                   // 8 bits
                function_instance: 0x0a,          // 5 bits
                ecu_instance: 0x05,               // 3 bits
                manufacturer_code: 0x02aa,        // 11 bits
                identity_number: 0x00155555,      // 21 bits
            },
        ];
        for i in &test_cases {
            let name = Name::new(
                i.arbitrary_address_capable,
                i.industry_group,
                i.vehicle_system_instance,
                i.vehicle_system,
                i.function,
                i.function_instance,
                i.ecu_instance,
                i.manufacturer_code,
                i.identity_number,
            );

            assert_eq!(
                i.arbitrary_address_capable,
                name.arbitrary_address_capable()
            );
            assert_eq!(i.industry_group, name.industry_group());
            assert_eq!(i.vehicle_system_instance, name.vehicle_system_instance());
            assert_eq!(i.vehicle_system, name.vehicle_system());
            assert_eq!(i.function, name.function());
            assert_eq!(i.function_instance, name.function_instance());
            assert_eq!(i.ecu_instance, name.ecu_instance());
            assert_eq!(i.manufacturer_code, name.manufacturer_code());
            assert_eq!(i.identity_number, name.identity_number());
        }
    }
}
