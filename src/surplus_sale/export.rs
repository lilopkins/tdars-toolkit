use crate::surplus_sale::types::ReconcileMethod;

use super::types::Datafile;

use bigdecimal::{BigDecimal, ToPrimitive, Zero};
use rust_xlsxwriter::{Format, FormatBorder, Formula, Workbook, XlsxError};

#[allow(
    clippy::unreadable_literal,
    reason = "hex colours are better not separated"
)]
const ALT_BG: u32 = 0xb4c7dc;
const COL_LOT: u16 = 1;
const COL_DESC: u16 = 2;
const COL_PARTY: u16 = 3;
const COL_METHOD: u16 = 4;
const COL_DEBIT: u16 = 5;
const COL_CREDIT: u16 = 6;
const COL_BAL: u16 = 7;

pub fn export(datafile: &Datafile) -> Result<Vec<u8>, XlsxError> {
    let mut workbook = Workbook::new();

    // Transactions
    create_transactions_sheet(&mut workbook, datafile)?;

    // Audit Log
    create_audit_sheet(&mut workbook, datafile)?;

    workbook.save_to_buffer()
}

#[allow(
    clippy::too_many_lines,
    reason = "this function encapsulates one behaviour"
)]
fn create_transactions_sheet(
    workbook: &mut Workbook,
    datafile: &Datafile,
) -> Result<(), XlsxError> {
    let title_format = Format::new().set_bold().set_font_size(28.);
    let table_heading_format = Format::new()
        .set_bold()
        .set_border_bottom(FormatBorder::Medium);
    let regular_format = Format::new();
    let alt_format = Format::new().set_background_color(ALT_BG);
    let open_closing_balance_format = Format::new().set_italic();
    let open_closing_balance_alt_format: Format = open_closing_balance_format
        .clone()
        .set_background_color(ALT_BG);

    let accounting_format = Format::new().set_num_format("[$£-809]#,##0.00;[RED]-[$£-809]#,##0.00");
    let accounting_alt_format = accounting_format.clone().set_background_color(ALT_BG);

    let worksheet = workbook
        .add_worksheet()
        .set_name("Transactions")?
        .set_screen_gridlines(false)
        .set_print_gridlines(false)
        .set_freeze_panes(4, 0)?
        .set_column_width(0, 3)?
        .set_column_width(COL_LOT, 13)?
        .set_column_width(COL_DESC, 35)?
        .set_column_width(COL_PARTY, 25)?
        .set_column_width(COL_METHOD, 20)?
        .set_column_width(COL_DEBIT, 10)?
        .set_column_width(COL_CREDIT, 10)?
        .set_column_width(COL_BAL, 10)?;

    worksheet.write_with_format(1, 1, "Transactions", &title_format)?;

    worksheet.write_with_format(3, COL_LOT, "Lot", &table_heading_format)?;
    worksheet.write_with_format(3, COL_DESC, "Description", &table_heading_format)?;
    worksheet.write_with_format(3, COL_PARTY, "Party", &table_heading_format)?;
    worksheet.write_with_format(3, COL_METHOD, "Method", &table_heading_format)?;
    worksheet.write_with_format(3, COL_DEBIT, "Debit", &table_heading_format)?;
    worksheet.write_with_format(3, COL_CREDIT, "Credit", &table_heading_format)?;
    worksheet.write_with_format(3, COL_BAL, "Balance", &table_heading_format)?;

    worksheet.write_with_format(4, COL_DESC, "Opening balance", &open_closing_balance_format)?;
    worksheet.write_with_format(4, COL_BAL, 0, &accounting_format)?;

    let mut row = 5;
    for item in datafile.items() {
        if let Some(sold) = &item.sold_details() {
            if let Some(method) = sold.buyer_reconciled() {
                let use_alt_format = row % 2 == 1;
                let fmt_reg = if use_alt_format {
                    &alt_format
                } else {
                    &regular_format
                };
                let fmt_acc = if use_alt_format {
                    &accounting_alt_format
                } else {
                    &accounting_format
                };

                worksheet.write_with_format(row, COL_LOT, item.lot_number(), fmt_reg)?;
                worksheet.write_with_format(row, COL_DESC, item.description(), fmt_reg)?;
                worksheet.write_with_format(
                    row,
                    COL_PARTY,
                    sold.buyer_callsign().to_string(),
                    fmt_reg,
                )?;
                worksheet.write_with_format(row, COL_DEBIT, "", fmt_reg)?;
                worksheet.write_with_format(row, COL_METHOD, method.to_string(), fmt_reg)?;
                worksheet.write_with_format(
                    row,
                    COL_CREDIT,
                    #[allow(clippy::unwrap_used, reason = "excel needs to deal with it!")]
                    sold.hammer_price().to_f64().unwrap(),
                    fmt_acc,
                )?;
                worksheet.write_with_format(
                    row,
                    COL_BAL,
                    Formula::new(format!("=H{}-F{}+G{}", row, row + 1, row + 1)),
                    fmt_acc,
                )?;

                row += 1;
            }

            if let Some(method) = sold.seller_reconciled() {
                let use_alt_format = row % 2 == 1;
                let fmt_reg = if use_alt_format {
                    &alt_format
                } else {
                    &regular_format
                };
                let fmt_acc = if use_alt_format {
                    &accounting_alt_format
                } else {
                    &accounting_format
                };

                worksheet.write_with_format(row, COL_LOT, item.lot_number(), fmt_reg)?;
                worksheet.write_with_format(row, COL_DESC, item.description(), fmt_reg)?;
                worksheet.write_with_format(
                    row,
                    COL_PARTY,
                    item.seller_callsign().to_string(),
                    fmt_reg,
                )?;
                worksheet.write_with_format(row, COL_METHOD, method.to_string(), fmt_reg)?;
                let hammer_less_club: BigDecimal =
                    if *sold.seller_reconciled() == Some(ReconcileMethod::Donation) {
                        BigDecimal::zero()
                    } else {
                        sold.hammer_price() * (1 - datafile.club_taking())
                    };
                worksheet.write_with_format(
                    row,
                    COL_DEBIT,
                    #[allow(clippy::unwrap_used, reason = "excel needs to deal with it!")]
                    hammer_less_club.to_f64().unwrap(),
                    fmt_acc,
                )?;
                worksheet.write_with_format(row, COL_CREDIT, "", fmt_reg)?;
                worksheet.write_with_format(
                    row,
                    COL_BAL,
                    Formula::new(format!("=H{}-F{}+G{}", row, row + 1, row + 1)),
                    fmt_acc,
                )?;

                row += 1;
            }
        }
    }

    for (cs, donation) in datafile.club_donations() {
        let use_alt_format = row % 2 == 1;
        let fmt_reg = if use_alt_format {
            &alt_format
        } else {
            &regular_format
        };
        let fmt_acc = if use_alt_format {
            &accounting_alt_format
        } else {
            &accounting_format
        };

        worksheet.write_with_format(row, COL_LOT, "", fmt_reg)?;
        worksheet.write_with_format(row, COL_DESC, "Club donation", fmt_reg)?;
        worksheet.write_with_format(row, COL_PARTY, cs.to_string(), fmt_reg)?;
        worksheet.write_with_format(row, COL_METHOD, "", fmt_reg)?;
        worksheet.write_with_format(row, COL_DEBIT, "", fmt_reg)?;
        worksheet.write_with_format(
            row,
            COL_CREDIT,
            #[allow(clippy::unwrap_used, reason = "excel needs to deal with it!")]
            donation.to_f64().unwrap(),
            fmt_acc,
        )?;
        worksheet.write_with_format(
            row,
            COL_BAL,
            Formula::new(format!("=H{}-F{}+G{}", row, row + 1, row + 1)),
            fmt_acc,
        )?;

        row += 1;
    }

    let use_alt_format = row % 2 == 1;
    let fmt_reg = if use_alt_format {
        &alt_format
    } else {
        &regular_format
    };
    let fmt_acc = if use_alt_format {
        &accounting_alt_format
    } else {
        &accounting_format
    };
    let fmt_open_close = if use_alt_format {
        &open_closing_balance_alt_format
    } else {
        &open_closing_balance_format
    };
    worksheet.write_with_format(row, COL_LOT, "", fmt_reg)?;
    worksheet.write_with_format(row, COL_DESC, "Closing balance", fmt_open_close)?;
    worksheet.write_with_format(row, COL_PARTY, "", fmt_reg)?;
    worksheet.write_with_format(row, COL_METHOD, "", fmt_reg)?;
    worksheet.write_with_format(row, COL_DEBIT, "", fmt_reg)?;
    worksheet.write_with_format(row, COL_CREDIT, "", fmt_reg)?;
    worksheet.write_with_format(row, COL_BAL, Formula::new(format!("=H{row}")), fmt_acc)?;

    Ok(())
}

fn create_audit_sheet(workbook: &mut Workbook, datafile: &Datafile) -> Result<(), XlsxError> {
    let title_format = Format::new().set_bold().set_font_size(28.);
    let table_heading_format = Format::new()
        .set_bold()
        .set_border_bottom(FormatBorder::Medium);
    let regular_format = Format::new();
    let alt_format = Format::new().set_background_color(ALT_BG);
    let audit_date_format = Format::new().set_num_format("YYYY-MM-DD HH:MM:SS.000");
    let audit_date_alt_format = audit_date_format.clone().set_background_color(ALT_BG);

    let worksheet = workbook
        .add_worksheet()
        .set_name("Audit Log")?
        .set_screen_gridlines(false)
        .set_print_gridlines(false)
        .set_freeze_panes(0, 5)?
        .set_column_width(0, 3)?
        .set_column_width(1, 25)?
        .set_column_width(2, 150)?;

    worksheet.write_with_format(1, 1, "Audit Log", &title_format)?;

    worksheet.write_with_format(3, 1, "Timestamp", &table_heading_format)?;
    worksheet.write_with_format(3, 2, "Event", &table_heading_format)?;

    for (idx, entry) in datafile.audit_log().iter().enumerate() {
        let use_alt_format = idx % 2 == 1;
        let fmt_reg = if use_alt_format {
            &alt_format
        } else {
            &regular_format
        };
        let fmt_date = if use_alt_format {
            &audit_date_alt_format
        } else {
            &audit_date_format
        };

        #[allow(
            clippy::unwrap_used,
            reason = "we want to panic if there are too many rows!"
        )]
        let row = (4 + idx).try_into().unwrap();
        worksheet.write_datetime_with_format(row, 1, entry.moment().naive_local(), fmt_date)?;
        worksheet.write_with_format(row, 2, format!("{}", entry.item()), fmt_reg)?;
    }

    Ok(())
}
