// Author: Quadri Atharu
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocumentType {
    Invoice,
    Receipt,
    BankStatement,
    PurchaseOrder,
    DeliveryNote,
    CreditNote,
    DebitNote,
    Unknown,
}

impl std::fmt::Display for DocumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentType::Invoice => write!(f, "Invoice"),
            DocumentType::Receipt => write!(f, "Receipt"),
            DocumentType::BankStatement => write!(f, "BankStatement"),
            DocumentType::PurchaseOrder => write!(f, "PurchaseOrder"),
            DocumentType::DeliveryNote => write!(f, "DeliveryNote"),
            DocumentType::CreditNote => write!(f, "CreditNote"),
            DocumentType::DebitNote => write!(f, "DebitNote"),
            DocumentType::Unknown => write!(f, "Unknown"),
        }
    }
}

pub fn classify(text: &str) -> DocumentType {
    let lower = text.to_lowercase();

    if matches_invoice(&lower) {
        return DocumentType::Invoice;
    }
    if matches_receipt(&lower) {
        return DocumentType::Receipt;
    }
    if matches_bank_statement(&lower) {
        return DocumentType::BankStatement;
    }
    if matches_purchase_order(&lower) {
        return DocumentType::PurchaseOrder;
    }
    if matches_debit_note(&lower) {
        return DocumentType::DebitNote;
    }
    if matches_delivery_note(&lower) {
        return DocumentType::DeliveryNote;
    }
    if matches_credit_note(&lower) {
        return DocumentType::CreditNote;
    }

    DocumentType::Unknown
}

fn matches_invoice(text: &str) -> bool {
    let keywords = ["invoice", "inv no", "inv number", "invoice no", "invoice number", "tax invoice"];
    keywords.iter().any(|k| text.contains(k))
}

fn matches_receipt(text: &str) -> bool {
    let keywords = ["receipt", "payment received", "received from", "cash receipt", "official receipt"];
    keywords.iter().any(|k| text.contains(k))
}

fn matches_bank_statement(text: &str) -> bool {
    let keywords = ["bank statement", "account statement", "statement of account", "bank account summary"];
    keywords.iter().any(|k| text.contains(k))
}

fn matches_purchase_order(text: &str) -> bool {
    let keywords = ["purchase order", "po number", "p.o. number", "purchase requisition", "order confirmation"];
    keywords.iter().any(|k| text.contains(k))
}

fn matches_delivery_note(text: &str) -> bool {
    let keywords = ["delivery note", "dn number", "delivery receipt", "despatch note", "goods received"];
    keywords.iter().any(|k| text.contains(k))
}

fn matches_credit_note(text: &str) -> bool {
    let keywords = ["credit note", "cn number", "credit memo", "credit adjustment"];
    keywords.iter().any(|k| text.contains(k))
}

fn matches_debit_note(text: &str) -> bool {
    let keywords = ["debit note", "debit memo", "debit adjustment"];
    if keywords.iter().any(|k| text.contains(k)) {
        return true;
    }
    if text.contains("dn") && (text.contains("debit") || text.contains("charge")) {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_invoice() {
        assert_eq!(
            classify("Tax Invoice INV-001 Date: 2024-01-15"),
            DocumentType::Invoice
        );
    }

    #[test]
    fn test_classify_receipt() {
        assert_eq!(
            classify("Payment received from John Doe Amount: 5000"),
            DocumentType::Receipt
        );
    }

    #[test]
    fn test_classify_bank_statement() {
        assert_eq!(
            classify("Account Statement for January 2024"),
            DocumentType::BankStatement
        );
    }

    #[test]
    fn test_classify_purchase_order() {
        assert_eq!(
            classify("Purchase Order PO-2024-0042"),
            DocumentType::PurchaseOrder
        );
    }

    #[test]
    fn test_classify_unknown() {
        assert_eq!(
            classify("Random document with no specific keywords"),
            DocumentType::Unknown
        );
    }
}
