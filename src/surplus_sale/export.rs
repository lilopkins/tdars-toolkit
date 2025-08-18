use super::types::*;

use bigdecimal::{BigDecimal, ToPrimitive};
use rust_xlsxwriter::*;

pub fn export(datafile: Datafile) -> Result<Vec<u8>, XlsxError> {
    let mut workbook = Workbook::new();

    let title_format = Format::new().set_bold().set_font_size(28.);
    let bold_format = Format::new().set_bold();
    let italic_format = Format::new().set_italic();
    let date_format = Format::new().set_num_format("YYYY-MM-DD HH:MM:SS.000");
    let accounting_format = Format::new().set_num_format("[$£-809]#,##0.00;[RED]-[$£-809]#,##0.00");

    // Transactions
    let worksheet = workbook
        .add_worksheet()
        .set_name("Transactions")?
        .set_screen_gridlines(false)
        .set_print_gridlines(false)
        .set_freeze_panes(0, 5)?
        .set_column_width(0, 3)?
        .set_column_width(1, 13)?
        .set_column_width(2, 35)?
        .set_column_width(3, 25)?
        .set_column_width(4, 10)?
        .set_column_width(5, 10)?
        .set_column_width(6, 10)?;

    worksheet.write_with_format(1, 1, "Transactions", &title_format)?;

    worksheet.write_with_format(3, 1, "Lot", &bold_format)?;
    worksheet.write_with_format(3, 2, "Description", &bold_format)?;
    worksheet.write_with_format(3, 3, "Party", &bold_format)?;
    worksheet.write_with_format(3, 4, "Debit", &bold_format)?;
    worksheet.write_with_format(3, 5, "Credit", &bold_format)?;
    worksheet.write_with_format(3, 6, "Balance", &bold_format)?;

    worksheet.write_with_format(4, 2, "Opening balance", &italic_format)?;
    worksheet.write_with_format(4, 6, 0, &accounting_format)?;

    let mut row = 5;
    for item in datafile.items() {
        if let Some(sold) = &item.sold_details() {
            if *sold.buyer_reconciled() {
                worksheet.write(row, 1, item.lot_number())?;
                worksheet.write(row, 2, item.description())?;
                worksheet.write(row, 3, sold.buyer_callsign().to_string())?;
                worksheet.write_with_format(
                    row,
                    5,
                    #[allow(clippy::unwrap_used, reason = "excel needs to deal with it!")]
                    sold.hammer_price().to_f64().unwrap(),
                    &accounting_format,
                )?;
                worksheet.write_with_format(
                    row,
                    6,
                    Formula::new(format!("=G{}-E{row}+F{row}", row - 1)),
                    &accounting_format,
                )?;

                row += 1;
            }

            if *sold.seller_reconciled() {
                worksheet.write(row, 1, item.lot_number())?;
                worksheet.write(row, 2, item.description())?;
                worksheet.write(row, 3, item.seller_callsign().to_string())?;
                let hammer_less_club: BigDecimal =
                    sold.hammer_price() * (1 - datafile.club_taking());
                worksheet.write_with_format(
                    row,
                    4,
                    #[allow(clippy::unwrap_used, reason = "excel needs to deal with it!")]
                    hammer_less_club.to_f64().unwrap(),
                    &accounting_format,
                )?;
                worksheet.write_with_format(
                    row,
                    6,
                    Formula::new(format!("=G{}-E{row}+F{row}", row - 1)),
                    &accounting_format,
                )?;

                row += 1;
            }
        }
    }

    worksheet.write_with_format(row, 2, "Closing balance", &italic_format)?;
    worksheet.write_with_format(row, 6, Formula::new(format!("=G{row}")), &accounting_format)?;

    // Audit Log
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

    worksheet.write_with_format(3, 1, "Timestamp", &bold_format)?;
    worksheet.write_with_format(3, 2, "Event", &bold_format)?;

    for (idx, entry) in datafile.audit_log().iter().enumerate() {
        #[allow(
            clippy::unwrap_used,
            reason = "we want to panic if there are too many rows!"
        )]
        let row = (4 + idx).try_into().unwrap();
        worksheet.write_datetime_with_format(row, 1, entry.moment().naive_local(), &date_format)?;
        worksheet.write(row, 2, format!("{}", entry.item()))?;
    }

    workbook.save_to_buffer()
}
