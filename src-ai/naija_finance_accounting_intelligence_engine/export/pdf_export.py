# Author: Quadri Atharu
"""PDF export engine using reportlab with invoice generation."""

from __future__ import annotations

import io
import os
from datetime import datetime
from typing import Any, Dict, List, Optional

from reportlab.lib import colors
from reportlab.lib.enums import TA_CENTER, TA_LEFT, TA_RIGHT
from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import ParagraphStyle, getSampleStyleSheet
from reportlab.lib.units import inch, mm
from reportlab.platypus import (
    HRFlowable,
    Paragraph,
    SimpleDocTemplate,
    Spacer,
    Table,
    TableStyle,
)
from reportlab.pdfgen import canvas

from ..core.logging import get_logger

logger = get_logger(__name__)

COMPANY_STYLE = ParagraphStyle(
    "CompanyName",
    parent=getSampleStyleSheet()["Heading1"],
    fontSize=18,
    textColor=colors.HexColor("#1F4E79"),
    spaceAfter=6,
)
TITLE_STYLE = ParagraphStyle(
    "DocTitle",
    parent=getSampleStyleSheet()["Heading2"],
    fontSize=14,
    textColor=colors.HexColor("#1F4E79"),
    spaceAfter=12,
    alignment=TA_CENTER,
)
NORMAL_STYLE = ParagraphStyle(
    "NormalCustom",
    parent=getSampleStyleSheet()["Normal"],
    fontSize=10,
    spaceAfter=4,
)
RIGHT_STYLE = ParagraphStyle(
    "RightAlign",
    parent=getSampleStyleSheet()["Normal"],
    fontSize=10,
    alignment=TA_RIGHT,
)
HEADER_BG = colors.HexColor("#1F4E79")
ROW_ALT_BG = colors.HexColor("#EBF5FB")


def export_to_pdf(
    data: Dict[str, Any],
    template: str = "default",
    filename: str = "report.pdf",
    output_dir: str = ".",
) -> str:
    filepath = os.path.join(output_dir, _ensure_extension(filename, ".pdf"))
    doc = SimpleDocTemplate(
        filepath,
        pagesize=A4,
        topMargin=0.75 * inch,
        bottomMargin=0.75 * inch,
        leftMargin=0.75 * inch,
        rightMargin=0.75 * inch,
    )

    elements: list = []

    if template == "invoice":
        elements.extend(_build_invoice_elements(data))
    elif template == "financial_statement":
        elements.extend(_build_financial_statement_elements(data))
    else:
        elements.extend(_build_default_elements(data))

    doc.build(elements)
    logger.info("pdf_exported", filename=filepath, template=template)
    return filepath


def generate_invoice_pdf(invoice_data: Dict[str, Any]) -> bytes:
    buffer = io.BytesIO()
    doc = SimpleDocTemplate(
        buffer,
        pagesize=A4,
        topMargin=0.5 * inch,
        bottomMargin=0.5 * inch,
        leftMargin=0.5 * inch,
        rightMargin=0.5 * inch,
    )
    elements = _build_invoice_elements(invoice_data)
    doc.build(elements)
    pdf_bytes = buffer.getvalue()
    buffer.close()
    logger.info("invoice_pdf_generated", invoice_number=invoice_data.get("invoice_number", ""))
    return pdf_bytes


def _build_invoice_elements(data: Dict[str, Any]) -> list:
    elements: list = []

    company_name = data.get("company_name", "HAQLY ERP")
    company_address = data.get("company_address", "")
    company_phone = data.get("company_phone", "")
    company_email = data.get("company_email", "")
    company_tin = data.get("company_tin", "")

    elements.append(Paragraph(company_name, COMPANY_STYLE))
    if company_address:
        elements.append(Paragraph(company_address, NORMAL_STYLE))
    contact_line = " | ".join(filter(None, [company_phone, company_email, f"TIN: {company_tin}" if company_tin else ""]))
    if contact_line:
        elements.append(Paragraph(contact_line, NORMAL_STYLE))

    elements.append(Spacer(1, 12))
    elements.append(HRFlowable(width="100%", thickness=2, color=HEADER_BG))
    elements.append(Spacer(1, 12))

    invoice_number = data.get("invoice_number", "")
    invoice_date = data.get("invoice_date", datetime.now().strftime("%d %B %Y"))
    due_date = data.get("due_date", "")

    header_data = [
        [Paragraph("<b>INVOICE</b>", TITLE_STYLE), "", "", ""],
        [f"Invoice No: {invoice_number}", "", f"Date: {invoice_date}", ""],
        [f"Due Date: {due_date}", "", "", ""],
    ]
    header_table = Table(header_data, colWidths=[2.5 * inch, 1 * inch, 2.5 * inch, 1 * inch])
    header_table.setStyle(TableStyle([
        ("SPAN", (0, 0), (-1, 0)),
        ("ALIGN", (0, 0), (-1, 0), "CENTER"),
        ("BOTTOMPADDING", (0, 0), (-1, 0), 12),
    ]))
    elements.append(header_table)
    elements.append(Spacer(1, 12))

    customer_name = data.get("customer_name", "")
    customer_address = data.get("customer_address", "")
    elements.append(Paragraph("<b>Bill To:</b>", NORMAL_STYLE))
    elements.append(Paragraph(customer_name, NORMAL_STYLE))
    if customer_address:
        elements.append(Paragraph(customer_address, NORMAL_STYLE))
    elements.append(Spacer(1, 12))

    line_items = data.get("line_items", [])
    if line_items:
        table_data = [
            [
                Paragraph("<b>#</b>", NORMAL_STYLE),
                Paragraph("<b>Description</b>", NORMAL_STYLE),
                Paragraph("<b>Qty</b>", NORMAL_STYLE),
                Paragraph("<b>Unit Price (NGN)</b>", NORMAL_STYLE),
                Paragraph("<b>Amount (NGN)</b>", NORMAL_STYLE),
            ]
        ]
        for idx, item in enumerate(line_items, 1):
            table_data.append([
                str(idx),
                str(item.get("description", "")),
                str(item.get("quantity", 1)),
                _fmt_ngn(float(item.get("unit_price", 0))),
                _fmt_ngn(float(item.get("amount", 0))),
            ])

        subtotal = sum(float(item.get("amount", 0)) for item in line_items)
        vat_rate = float(data.get("vat_rate", 0.075))
        vat_amount = round(subtotal * vat_rate, 2)
        total = round(subtotal + vat_amount, 2)

        table_data.append(["", "", "", Paragraph("<b>Subtotal</b>", NORMAL_STYLE), _fmt_ngn(subtotal)])
        table_data.append(["", "", "", Paragraph("<b>VAT (7.5%)</b>", NORMAL_STYLE), _fmt_ngn(vat_amount)])
        table_data.append(["", "", "", Paragraph("<b>TOTAL</b>", NORMAL_STYLE), _fmt_ngn(total)])

        col_widths = [0.4 * inch, 2.8 * inch, 0.6 * inch, 1.3 * inch, 1.3 * inch]
        inv_table = Table(table_data, colWidths=col_widths)

        style_cmds = [
            ("BACKGROUND", (0, 0), (-1, 0), HEADER_BG),
            ("TEXTCOLOR", (0, 0), (-1, 0), colors.white),
            ("FONTNAME", (0, 0), (-1, 0), "Helvetica-Bold"),
            ("FONTSIZE", (0, 0), (-1, 0), 10),
            ("ALIGN", (0, 0), (0, -1), "CENTER"),
            ("ALIGN", (2, 0), (2, -1), "CENTER"),
            ("ALIGN", (3, 0), (4, -1), "RIGHT"),
            ("GRID", (0, 0), (-1, -3), 0.5, colors.grey),
            ("BOTTOMPADDING", (0, 0), (-1, 0), 8),
            ("TOPPADDING", (0, 0), (-1, 0), 8),
            ("LINEABOVE", (0, -3), (-1, -3), 1, HEADER_BG),
            ("LINEABOVE", (0, -2), (-1, -2), 0.5, colors.grey),
            ("LINEABOVE", (0, -1), (-1, -1), 1.5, HEADER_BG),
            ("FONTNAME", (0, -1), (-1, -1), "Helvetica-Bold"),
        ]
        for i in range(1, len(line_items) + 1):
            if i % 2 == 0:
                style_cmds.append(("BACKGROUND", (0, i), (-1, i), ROW_ALT_BG))

        inv_table.setStyle(TableStyle(style_cmds))
        elements.append(inv_table)

    payment_terms = data.get("payment_terms", "")
    if payment_terms:
        elements.append(Spacer(1, 12))
        elements.append(Paragraph(f"<b>Payment Terms:</b> {payment_terms}", NORMAL_STYLE))

    bank_details = data.get("bank_details")
    if bank_details:
        elements.append(Spacer(1, 12))
        elements.append(Paragraph("<b>Bank Details:</b>", NORMAL_STYLE))
        elements.append(Paragraph(f"Bank: {bank_details.get('bank_name', '')}", NORMAL_STYLE))
        elements.append(Paragraph(f"Account Name: {bank_details.get('account_name', '')}", NORMAL_STYLE))
        elements.append(Paragraph(f"Account Number: {bank_details.get('account_number', '')}", NORMAL_STYLE))
        elements.append(Paragraph(f"Sort Code: {bank_details.get('sort_code', '')}", NORMAL_STYLE))

    return elements


def _build_financial_statement_elements(data: Dict[str, Any]) -> list:
    elements: list = []
    title = data.get("title", "Financial Statement")
    company_name = data.get("company_name", "")
    period = data.get("period", "")

    elements.append(Paragraph(company_name, COMPANY_STYLE))
    elements.append(Paragraph(title, TITLE_STYLE))
    if period:
        elements.append(Paragraph(f"For the period: {period}", NORMAL_STYLE))
    elements.append(HRFlowable(width="100%", thickness=1.5, color=HEADER_BG))
    elements.append(Spacer(1, 12))

    sections = data.get("sections", [])
    for section in sections:
        section_title = section.get("title", "")
        if section_title:
            elements.append(Paragraph(f"<b>{section_title}</b>", NORMAL_STYLE))

        rows = section.get("rows", [])
        if rows:
            table_data = []
            for row in rows:
                label = row.get("label", "")
                amount = row.get("amount", 0)
                indent = int(row.get("indent", 0))
                is_total = row.get("is_total", False)
                prefix = "    " * indent
                styled_label = f"<b>{prefix}{label}</b>" if is_total else f"{prefix}{label}"
                table_data.append([
                    Paragraph(styled_label, NORMAL_STYLE),
                    Paragraph(_fmt_ngn(amount), RIGHT_STYLE if not is_total else ParagraphStyle("BoldRight", parent=RIGHT_STYLE, fontName="Helvetica-Bold")),
                ])

            if table_data:
                tbl = Table(table_data, colWidths=[4 * inch, 2.5 * inch])
                style_cmds = [
                    ("ALIGN", (1, 0), (1, -1), "RIGHT"),
                    ("BOTTOMPADDING", (0, 0), (-1, -1), 4),
                    ("TOPPADDING", (0, 0), (-1, -1), 2),
                ]
                for i, row in enumerate(rows):
                    if row.get("is_total"):
                        style_cmds.append(("LINEABOVE", (0, i), (-1, i), 0.5, colors.black))
                        style_cmds.append(("LINEBELOW", (0, i), (-1, i), 1, colors.black))

                tbl.setStyle(TableStyle(style_cmds))
                elements.append(tbl)
                elements.append(Spacer(1, 8))

    return elements


def _build_default_elements(data: Dict[str, Any]) -> list:
    elements: list = []
    title = data.get("title", "Report")
    generated_at = data.get("generated_at", datetime.now().strftime("%d %B %Y, %H:%M"))

    elements.append(Paragraph(title, TITLE_STYLE))
    elements.append(Paragraph(f"Generated: {generated_at}", NORMAL_STYLE))
    elements.append(HRFlowable(width="100%", thickness=1, color=HEADER_BG))
    elements.append(Spacer(1, 12))

    rows = data.get("rows", data.get("data", []))
    if rows and isinstance(rows, list) and len(rows) > 0:
        if isinstance(rows[0], dict):
            headers = list(rows[0].keys())
            table_data = [[Paragraph(f"<b>{h.replace('_', ' ').title()}</b>", NORMAL_STYLE) for h in headers]]
            for row in rows:
                table_data.append([
                    Paragraph(str(row.get(h, "")), NORMAL_STYLE) for h in headers
                ])
            num_cols = len(headers)
            col_width = 6.5 * inch / max(num_cols, 1)
            tbl = Table(table_data, colWidths=[col_width] * num_cols)
            tbl.setStyle(TableStyle([
                ("BACKGROUND", (0, 0), (-1, 0), HEADER_BG),
                ("TEXTCOLOR", (0, 0), (-1, 0), colors.white),
                ("GRID", (0, 0), (-1, -1), 0.5, colors.grey),
                ("FONTNAME", (0, 0), (-1, 0), "Helvetica-Bold"),
                ("BOTTOMPADDING", (0, 0), (-1, 0), 8),
            ]))
            elements.append(tbl)

    return elements


def _fmt_ngn(value: float) -> str:
    if value == 0:
        return "0.00"
    sign = "-" if value < 0 else ""
    return f"{sign}{abs(value):,.2f}"


def _ensure_extension(filename: str, ext: str) -> str:
    if not filename.lower().endswith(ext):
        return filename + ext
    return filename
