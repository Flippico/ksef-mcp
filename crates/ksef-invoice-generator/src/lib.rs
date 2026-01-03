use chrono::{DateTime, Local};
use serde::Serialize;

/// Represents a party (buyer or seller) in the invoice
#[derive(Debug, Clone, Serialize)]
pub struct Party {
    /// Tax identification number (NIP)
    #[serde(rename = "NIP")]
    pub nip: String,
    /// Name of the party
    #[serde(rename = "Nazwa")]
    pub nazwa: String,
    /// Address of the party
    #[serde(rename = "AdresL1", skip_serializing_if = "Option::is_none")]
    pub adres: Option<String>,
}

/// Represents a line item in the invoice
#[derive(Debug, Clone, Serialize)]
pub struct InvoiceLineItem {
    /// Line number
    #[serde(rename = "NrWierszaFa")]
    pub nr_wiersza: u32,
    /// Product/service description
    #[serde(rename = "P_7")]
    pub opis: String,
    /// Unit of measurement
    #[serde(rename = "P_8A")]
    pub jednostka: String,
    /// Quantity
    #[serde(rename = "P_8B")]
    pub ilosc: f64,
    /// Net unit price
    #[serde(rename = "P_9A")]
    pub cena_netto: f64,
    /// Net amount (quantity * unit price)
    #[serde(rename = "P_11")]
    pub kwota_netto: f64,
    /// VAT rate percentage
    #[serde(rename = "P_12")]
    pub stawka_vat: u8,
}

/// Main invoice structure
#[derive(Debug, Clone)]
pub struct Invoice {
    /// Seller (Podmiot1)
    pub sprzedawca: Party,
    /// Buyer (Podmiot2)
    pub nabywca: Party,
    /// Invoice line items
    pub pozycje: Vec<InvoiceLineItem>,
    /// Invoice date
    pub data_wystawienia: String,
    /// Invoice number
    pub numer: String,
    /// Currency code (default: PLN)
    pub waluta: String,
}

impl Invoice {
    /// Creates a new invoice
    pub fn new(
        sprzedawca: Party,
        nabywca: Party,
        data_wystawienia: String,
        numer: String,
    ) -> Self {
        Self {
            sprzedawca,
            nabywca,
            pozycje: Vec::new(),
            data_wystawienia,
            numer,
            waluta: "PLN".to_string(),
        }
    }

    /// Adds a line item to the invoice
    pub fn add_line_item(&mut self, item: InvoiceLineItem) {
        self.pozycje.push(item);
    }

    /// Calculates total net amount
    pub fn calculate_total_net(&self) -> f64 {
        self.pozycje.iter().map(|p| p.kwota_netto).sum()
    }

    /// Calculates total VAT amount
    pub fn calculate_total_vat(&self) -> f64 {
        self.pozycje
            .iter()
            .map(|p| p.kwota_netto * (p.stawka_vat as f64 / 100.0))
            .sum()
    }

    /// Calculates total gross amount
    pub fn calculate_total_gross(&self) -> f64 {
        self.calculate_total_net() + self.calculate_total_vat()
    }

    /// Generates KSeF 2.0 compliant XML for the invoice
    ///
    /// This generates an FA(2) structured VAT invoice according to the KSeF 2.0 format.
    /// The XML follows the schema: http://crd.gov.pl/wzor/2023/06/29/12648/
    ///
    /// # Returns
    ///
    /// A String containing the complete XML document ready to be sent to KSeF 2.0
    ///
    /// # Example
    ///
    /// ```
    /// use ksef_invoice_generator::{Invoice, Party, InvoiceLineItem};
    ///
    /// let seller = Party {
    ///     nip: "1234567890".to_string(),
    ///     nazwa: "Example Company Sp. z o.o.".to_string(),
    ///     adres: Some("ul. Testowa 1, 00-001 Warszawa".to_string()),
    /// };
    ///
    /// let buyer = Party {
    ///     nip: "9876543210".to_string(),
    ///     nazwa: "Buyer Company Sp. z o.o.".to_string(),
    ///     adres: Some("ul. Kupiecka 2, 00-002 Warszawa".to_string()),
    /// };
    ///
    /// let mut invoice = Invoice::new(
    ///     seller,
    ///     buyer,
    ///     "2026-01-03".to_string(),
    ///     "FV/2026/01/001".to_string(),
    /// );
    ///
    /// let item = InvoiceLineItem {
    ///     nr_wiersza: 1,
    ///     opis: "Usługa konsultingowa".to_string(),
    ///     jednostka: "szt".to_string(),
    ///     ilosc: 1.0,
    ///     cena_netto: 1000.0,
    ///     kwota_netto: 1000.0,
    ///     stawka_vat: 23,
    /// };
    ///
    /// invoice.add_line_item(item);
    ///
    /// let xml = invoice.generate_ksef_xml();
    /// ```
    pub fn generate_ksef_xml(&self) -> String {
        let now: DateTime<Local> = Local::now();
        let data_wytworzenia = now.to_rfc3339();

        let total_net = self.calculate_total_net();
        let total_vat = self.calculate_total_vat();
        let total_gross = self.calculate_total_gross();

        // Build line items XML
        let mut line_items_xml = String::new();
        for item in &self.pozycje {
            line_items_xml.push_str(&format!(
                r#"    <FaWiersz>
      <NrWierszaFa>{}</NrWierszaFa>
      <P_7>{}</P_7>
      <P_8A>{}</P_8A>
      <P_8B>{}</P_8B>
      <P_9A>{:.2}</P_9A>
      <P_11>{:.2}</P_11>
      <P_12>{}</P_12>
    </FaWiersz>
"#,
                item.nr_wiersza,
                escape_xml(&item.opis),
                escape_xml(&item.jednostka),
                item.ilosc,
                item.cena_netto,
                item.kwota_netto,
                item.stawka_vat
            ));
        }

        // Build seller address XML
        let sprzedawca_adres_xml = if let Some(ref adres) = self.sprzedawca.adres {
            format!(
                r#"    <Adres>
      <KodKraju>PL</KodKraju>
      <AdresL1>{}</AdresL1>
    </Adres>"#,
                escape_xml(adres)
            )
        } else {
            String::new()
        };

        // Generate complete XML document
        format!(
            r#"<?xml version="1.0" encoding="utf-8" standalone="yes"?>
<Faktura
    xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
    xmlns:xsd="http://www.w3.org/2001/XMLSchema"
    xmlns="http://crd.gov.pl/wzor/2023/06/29/12648/">
  <Naglowek>
    <KodFormularza kodSystemowy="FA (2)" wersjaSchemy="1-0E">FA</KodFormularza>
    <WariantFormularza>2</WariantFormularza>
    <DataWytworzeniaFa>{}</DataWytworzeniaFa>
    <SystemInfo>KSeF Rust Client 1.0</SystemInfo>
  </Naglowek>
  <Podmiot1>
    <DaneIdentyfikacyjne>
      <NIP>{}</NIP>
      <Nazwa>{}</Nazwa>
    </DaneIdentyfikacyjne>
{}
  </Podmiot1>
  <Podmiot2>
    <DaneIdentyfikacyjne>
      <NIP>{}</NIP>
      <Nazwa>{}</Nazwa>
    </DaneIdentyfikacyjne>
    <NrKlienta>2</NrKlienta>
    <JST>2</JST>
    <GV>2</GV>
  </Podmiot2>
  <Fa>
    <KodWaluty>{}</KodWaluty>
    <P_1>{}</P_1>
    <P_1M>dom</P_1M>
    <P_2>{}</P_2>
    <P_13_1>{:.2}</P_13_1>
    <P_14_1>{:.2}</P_14_1>
    <P_15>{:.2}</P_15>
    <Adnotacje>
      <P_16>2</P_16>
      <P_17>2</P_17>
      <P_18>2</P_18>
      <P_18A>2</P_18A>
      <Zwolnienie>
        <P_19N>1</P_19N>
      </Zwolnienie>
      <NoweSrodkiTransportu>
        <P_22N>1</P_22N>
      </NoweSrodkiTransportu>
      <P_23>2</P_23>
      <PMarzy>
        <P_PMarzyN>1</P_PMarzyN>
      </PMarzy>
    </Adnotacje>
    <RodzajFaktury>VAT</RodzajFaktury>
{}  </Fa>
</Faktura>"#,
            data_wytworzenia,
            self.sprzedawca.nip,
            escape_xml(&self.sprzedawca.nazwa),
            sprzedawca_adres_xml,
            self.nabywca.nip,
            escape_xml(&self.nabywca.nazwa),
            self.waluta,
            self.data_wystawienia,
            escape_xml(&self.numer),
            total_net,
            total_vat,
            total_gross,
            line_items_xml
        )
    }
}

/// Helper function to escape XML special characters
fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invoice_creation() {
        let seller = Party {
            nip: "1234567890".to_string(),
            nazwa: "Test Seller".to_string(),
            adres: Some("Test Address 1".to_string()),
        };

        let buyer = Party {
            nip: "9876543210".to_string(),
            nazwa: "Test Buyer".to_string(),
            adres: None,
        };

        let invoice = Invoice::new(seller, buyer, "2026-01-03".to_string(), "FV/1/2026".to_string());

        assert_eq!(invoice.sprzedawca.nip, "1234567890");
        assert_eq!(invoice.nabywca.nip, "9876543210");
        assert_eq!(invoice.data_wystawienia, "2026-01-03");
        assert_eq!(invoice.numer, "FV/1/2026");
    }

    #[test]
    fn test_add_line_item() {
        let seller = Party {
            nip: "1234567890".to_string(),
            nazwa: "Test Seller".to_string(),
            adres: Some("Test Address".to_string()),
        };

        let buyer = Party {
            nip: "9876543210".to_string(),
            nazwa: "Test Buyer".to_string(),
            adres: None,
        };

        let mut invoice = Invoice::new(seller, buyer, "2026-01-03".to_string(), "FV/1/2026".to_string());

        let item = InvoiceLineItem {
            nr_wiersza: 1,
            opis: "Test Item".to_string(),
            jednostka: "szt".to_string(),
            ilosc: 2.0,
            cena_netto: 100.0,
            kwota_netto: 200.0,
            stawka_vat: 23,
        };

        invoice.add_line_item(item);

        assert_eq!(invoice.pozycje.len(), 1);
        assert_eq!(invoice.pozycje[0].opis, "Test Item");
    }

    #[test]
    fn test_calculate_totals() {
        let seller = Party {
            nip: "1234567890".to_string(),
            nazwa: "Test Seller".to_string(),
            adres: Some("Test Address".to_string()),
        };

        let buyer = Party {
            nip: "9876543210".to_string(),
            nazwa: "Test Buyer".to_string(),
            adres: None,
        };

        let mut invoice = Invoice::new(seller, buyer, "2026-01-03".to_string(), "FV/1/2026".to_string());

        let item1 = InvoiceLineItem {
            nr_wiersza: 1,
            opis: "Item 1".to_string(),
            jednostka: "szt".to_string(),
            ilosc: 2.0,
            cena_netto: 100.0,
            kwota_netto: 200.0,
            stawka_vat: 23,
        };

        let item2 = InvoiceLineItem {
            nr_wiersza: 2,
            opis: "Item 2".to_string(),
            jednostka: "szt".to_string(),
            ilosc: 1.0,
            cena_netto: 300.0,
            kwota_netto: 300.0,
            stawka_vat: 23,
        };

        invoice.add_line_item(item1);
        invoice.add_line_item(item2);

        assert_eq!(invoice.calculate_total_net(), 500.0);
        assert_eq!(invoice.calculate_total_vat(), 115.0);
        assert_eq!(invoice.calculate_total_gross(), 615.0);
    }

    #[test]
    fn test_generate_xml() {
        let seller = Party {
            nip: "1234567890".to_string(),
            nazwa: "Example Company".to_string(),
            adres: Some("ul. Testowa 1".to_string()),
        };

        let buyer = Party {
            nip: "9876543210".to_string(),
            nazwa: "Buyer Company".to_string(),
            adres: None,
        };

        let mut invoice = Invoice::new(seller, buyer, "2026-01-03".to_string(), "FV/1/2026".to_string());

        let item = InvoiceLineItem {
            nr_wiersza: 1,
            opis: "Test Service".to_string(),
            jednostka: "szt".to_string(),
            ilosc: 1.0,
            cena_netto: 1000.0,
            kwota_netto: 1000.0,
            stawka_vat: 23,
        };

        invoice.add_line_item(item);

        let xml = invoice.generate_ksef_xml();

        assert!(xml.contains("<?xml version=\"1.0\" encoding=\"utf-8\""));
        assert!(xml.contains("<Faktura"));
        assert!(xml.contains("xmlns=\"http://crd.gov.pl/wzor/2023/06/29/12648/\""));
        assert!(xml.contains("<NIP>1234567890</NIP>"));
        assert!(xml.contains("<Nazwa>Example Company</Nazwa>"));
        assert!(xml.contains("<P_15>1230.00</P_15>"));
    }

    #[test]
    fn test_xml_escaping() {
        assert_eq!(escape_xml("Test & <tag>"), "Test &amp; &lt;tag&gt;");
        assert_eq!(escape_xml("Quote \"test\""), "Quote &quot;test&quot;");
        assert_eq!(escape_xml("Apostrophe 'test'"), "Apostrophe &apos;test&apos;");
    }
}
