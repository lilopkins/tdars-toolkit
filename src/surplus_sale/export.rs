use super::types::Datafile;

use bigdecimal::{BigDecimal, ToPrimitive, Zero};
use rust_xlsxwriter::{Format, FormatBorder, Formula, Workbook, XlsxError};

#[allow(
    clippy::unreadable_literal,
    reason = "hex colours are better not separated"
)]
const ALT_BG: u32 = 0xb4c7dc;

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
        .set_column_width(1, 13)?
        .set_column_width(2, 35)?
        .set_column_width(3, 25)?
        .set_column_width(4, 10)?
        .set_column_width(5, 10)?
        .set_column_width(6, 10)?;

    worksheet.write_with_format(1, 1, "Transactions", &title_format)?;

    worksheet.write_with_format(3, 1, "Lot", &table_heading_format)?;
    worksheet.write_with_format(3, 2, "Description", &table_heading_format)?;
    worksheet.write_with_format(3, 3, "Party", &table_heading_format)?;
    worksheet.write_with_format(3, 4, "Debit", &table_heading_format)?;
    worksheet.write_with_format(3, 5, "Credit", &table_heading_format)?;
    worksheet.write_with_format(3, 6, "Balance", &table_heading_format)?;

    worksheet.write_with_format(4, 2, "Opening balance", &open_closing_balance_format)?;
    worksheet.write_with_format(4, 6, 0, &accounting_format)?;

    let mut row = 5;
    for item in datafile.items() {
        if let Some(sold) = &item.sold_details() {
            if *sold.buyer_reconciled() {
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

                worksheet.write_with_format(row, 1, item.lot_number(), fmt_reg)?;
                worksheet.write_with_format(row, 2, item.description(), fmt_reg)?;
                worksheet.write_with_format(row, 3, sold.buyer_callsign().to_string(), fmt_reg)?;
                worksheet.write_with_format(row, 4, "", fmt_reg)?;
                worksheet.write_with_format(
                    row,
                    5,
                    #[allow(clippy::unwrap_used, reason = "excel needs to deal with it!")]
                    sold.hammer_price().to_f64().unwrap(),
                    fmt_acc,
                )?;
                worksheet.write_with_format(
                    row,
                    6,
                    Formula::new(format!("=G{}-E{}+F{}", row, row + 1, row + 1)),
                    fmt_acc,
                )?;

                row += 1;
            }

            if *sold.seller_reconciled() {
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

                worksheet.write_with_format(row, 1, item.lot_number(), fmt_reg)?;
                if *sold.seller_all_funds_to_club() {
                    worksheet.write_with_format(
                        row,
                        2,
                        format!("{} (seller donated funds to club)", item.description()),
                        fmt_reg,
                    )?;
                } else {
                    worksheet.write_with_format(row, 2, item.description(), fmt_reg)?;
                }
                worksheet.write_with_format(row, 3, item.seller_callsign().to_string(), fmt_reg)?;
                let hammer_less_club: BigDecimal = if *sold.seller_all_funds_to_club() {
                    BigDecimal::zero()
                } else {
                    sold.hammer_price() * (1 - datafile.club_taking())
                };
                worksheet.write_with_format(
                    row,
                    4,
                    #[allow(clippy::unwrap_used, reason = "excel needs to deal with it!")]
                    hammer_less_club.to_f64().unwrap(),
                    fmt_acc,
                )?;
                worksheet.write_with_format(row, 5, "", fmt_reg)?;
                worksheet.write_with_format(
                    row,
                    6,
                    Formula::new(format!("=G{}-E{}+F{}", row, row + 1, row + 1)),
                    fmt_acc,
                )?;

                row += 1;
            }
        }
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
    worksheet.write_with_format(row, 1, "", fmt_reg)?;
    worksheet.write_with_format(row, 2, "Closing balance", fmt_open_close)?;
    worksheet.write_with_format(row, 3, "", fmt_reg)?;
    worksheet.write_with_format(row, 4, "", fmt_reg)?;
    worksheet.write_with_format(row, 5, "", fmt_reg)?;
    worksheet.write_with_format(row, 6, Formula::new(format!("=G{}", row - 1)), fmt_acc)?;

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
