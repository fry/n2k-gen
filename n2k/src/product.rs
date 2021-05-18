pub struct Product<'a> {
    n2k: u8,
    code: u8,
    model: &'a str,
    software: &'a str,
    version: &'a str,
    serial: &'a str,
    certification: u8,
    load: u8,
}

impl<'a> Product<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        n2k: u8,
        code: u8,
        model: &'a str,
        software: &'a str,
        version: &'a str,
        serial: &'a str,
        certification: u8,
        load: u8,
    ) -> Self {
        //TODO: validate parameters

        Product {
            n2k,
            code,
            model,
            software,
            version,
            serial,
            certification,
            load,
        }
    }

    pub fn n2k(&self) -> u8 {
        self.n2k
    }

    pub fn code(&self) -> u8 {
        self.code
    }

    pub fn model(&self) -> &'a str {
        self.model
    }

    pub fn software(&self) -> &'a str {
        self.software
    }

    pub fn version(&self) -> &'a str {
        self.version
    }

    pub fn serial(&self) -> &'a str {
        self.serial
    }

    pub fn certification(&self) -> u8 {
        self.certification
    }

    pub fn load(&self) -> u8 {
        self.load
    }
}
